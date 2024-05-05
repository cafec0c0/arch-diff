use std::cmp::Ordering;
use std::collections::HashMap;
use std::env::consts::ARCH;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path};
use bytes::Buf;
use flate2::read::GzDecoder;
use tar::EntryType;

#[derive(Debug)]
struct Package {
    name: String,
    version: String,
    repo: String
}

impl Package {
    fn new(name: String, version: String, repo: String) -> Self {
        Package {
            name,
            version,
            repo
        }
    }

    fn default() -> Self {
        Package::new(String::from("none"), String::from("none"), String::from("none"))
    }
}

fn read_mirror_list(path: String) -> Result<Vec<String>, String> {
    let path = Path::new(&path);

    if !path.exists() {
        return Err(String::from("mirrorlist does not exist"))
    }

    let contents = fs::read_to_string(path).unwrap();

    Ok(contents.lines()
        .filter(|line| line.starts_with("Server"))
        .map(|line| line.split_whitespace().last().unwrap())
        .map(|line| line.into())
        .collect())
}

fn substitute_url_vars(url: &String, repo: &String) -> String {
    url
        .replace("$repo", repo)
        .replace("$arch", ARCH)
}

fn read_package_desc(data: String) -> Result<Package, String> {
    let mut package = Package::default();

    let lines: Vec<String> = data.lines().map(|s| String::from(s)).collect();

    for (idx, line) in lines.iter().enumerate() {
        if line.contains("%NAME%") {
            package.name = lines[idx + 1].to_owned()
        }
        if line.contains("%VERSION%") {
            package.version = lines[idx + 1].to_owned()
        }
    }

    Ok(package)
}

fn get_local_packages() -> HashMap<String, Package> {
    let local_repo_dir = "/var/lib/pacman/local";

    let paths = fs::read_dir(local_repo_dir).unwrap();

    paths
        .map(|path| path.unwrap().path().join("desc"))
        .filter(|path| Path::new(path).exists())
        .map(|path| read_package_desc(fs::read_to_string(path).unwrap()))
        .filter(|pkg| pkg.is_ok())
        .map(|pkg| pkg.unwrap())
        .map(|pkg| (pkg.name.to_owned(), pkg))
        .collect()
}

#[derive(Debug)]
struct Repository {
    name: String,
    servers: Vec<String>
}

fn get_local_repositories() -> Vec<Repository> {
    let path = Path::new("/etc/pacman.conf");

    let lines2 = fs::read_to_string(path).unwrap();

    let lines = lines2.lines();

    let mut repos = vec![];

    // current repo data
    let mut name = "";
    let mut servers: Vec<String> = vec![];
    let mut first = true;

    let line_count = lines.to_owned().count();

    for (idx, line) in lines.enumerate() {
        if line.starts_with("[options]") {
            continue;
        }

        if idx == line_count - 1 {
            repos.push(Repository { name: name.to_owned(), servers: servers.to_owned() });
        }

        if line.starts_with("[") {
            if !first {
                repos.push(Repository { name: name.to_owned(), servers: servers.to_owned() });
            }

            name = line.split_once("[").unwrap().1.split_once("]").unwrap().0.into();
            servers = vec![];
            first = false;
        }

        if line.starts_with("Server") {
            servers.push(line.split_whitespace().last().unwrap().into());
        }

        if line.starts_with("Include") {
            // We should probably parse this recursively, but we'll just assume it's a list of mirrors
            read_mirror_list(line.split_whitespace().last().unwrap().into())
                .unwrap()
                .iter()
                .for_each(|s| servers.push(s.to_owned()))
        }
    }

    repos
}

fn get_remote_database(repo: &Repository) -> Result<bytes::Bytes, String>{
    for server in &repo.to_owned().servers {
        let url = format!("{}/{}.db", substitute_url_vars(&server, &repo.name), &repo.name);

        let resp = reqwest::blocking::get(url);
        if resp.is_ok() {
            return Ok(resp.unwrap().bytes().unwrap())
        }
    }

    Err(String::from("unable to fetch database from any servers"))
}

fn get_remote_packages() -> HashMap<String, Package> {
    let repos = get_local_repositories();

    let mut packages = HashMap::new();

    for repo in repos {
        println!(":: Fetching package database for {}", repo.name);
        let database_file = get_remote_database(&repo);
        let reader = BufReader::new(database_file.unwrap().reader());
        let decoder = GzDecoder::new(reader);

        let mut binding = tar::Archive::new(decoder);
        let entries = binding.entries().unwrap();

        for entry in entries {
            let unwrapped = &mut entry.unwrap();

            if unwrapped.header().entry_type() == EntryType::Regular && unwrapped.path().unwrap().ends_with("desc") {
                let mut s: String = String::new();
                unwrapped.read_to_string(&mut s).unwrap();

                let mut p = read_package_desc(s).unwrap();
                let name = p.name.to_owned();
                p.repo = repo.name.to_owned();
                packages.insert(name, p);
            }
        }
    }

    packages
}

#[derive(PartialEq)]
enum PackageComparison {
    Same,
    Different
}

fn compare_package_versions(pkg1: &Package, pkg2: &Package) -> PackageComparison {
    match pkg1.version == pkg2.version {
        true => {
            PackageComparison::Same
        }
        false => {
            PackageComparison::Different
        }
    }
}

struct PackagePair {
    pkg1: Package,
    pkg2: Package
}

fn main() {
    let local_packages = get_local_packages();
    let remote_packages = get_remote_packages();

    println!(":: Comparing package versions");

    let mut longest_name_len = 0;
    let mut longest_version = 0;
    let mut count = 0;
    for pkg in &local_packages {
        if remote_packages.get(pkg.0).is_some() {
            let total_len = pkg.0.len() + remote_packages.get(pkg.0).unwrap().repo.len() + 1;
            if total_len > longest_name_len && pkg.1.version !=  remote_packages.get(pkg.0).unwrap().version {
                longest_name_len = total_len
            }
        }

        if remote_packages.get(pkg.0).is_some() && pkg.1.version.len() > longest_version && pkg.1.version !=  remote_packages.get(pkg.0).unwrap().version {
           longest_version = pkg.1.version.len();
        }

        if remote_packages.get(pkg.0).is_some() && pkg.1.version !=  remote_packages.get(pkg.0).unwrap().version {
            count += 1;
        }
    }

    println!(":: {} packages to upgrade", count);

    let mut outdated_packages = vec![];

    for package in local_packages {
        if remote_packages.contains_key(&package.0) {
            let remote_package = remote_packages.get(&package.0).unwrap();
            if compare_package_versions(&package.1, &remote_package) == PackageComparison::Different {
                let other = remote_package.to_owned();
                outdated_packages.push(PackagePair{pkg1: package.1, pkg2: Package {
                    name: other.name.to_owned(),
                    version: other.version.to_owned(),
                    repo: other.repo.to_owned()
                } });
                // let package_id = format!("{}/{}", &remote_package.repo, &remote_package.name);
                // println!("{:<width$} {:<ver_width$} -> {}", package_id, &package.1.version, &remote_package.version, width = longest_name_len + 1, ver_width = longest_version)
            }
        }
    }

    outdated_packages.sort_by(|a,b| {
        let cmp_res = a.pkg2.repo.partial_cmp(&b.pkg2.repo).unwrap();
        if cmp_res != Ordering::Equal {
            return cmp_res;
        }

        a.pkg2.name.partial_cmp(&b.pkg2.name).unwrap()
    });

    for x in outdated_packages {
        let package_id = format!("{}/{}", &x.pkg2.repo, &x.pkg2.name);
        println!("{:<width$} {:<ver_width$} -> {}", package_id, &x.pkg1.version, &x.pkg2.version, width = longest_name_len + 1, ver_width = longest_version)
    }
}
