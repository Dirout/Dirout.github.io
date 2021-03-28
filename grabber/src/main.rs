use clap::clap_app;
use octocrab::params;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Repository {
    archived: bool,
    forked: bool,
    title: String,
    description: String,
    has_issues: bool,
    has_license: bool,
    html_url: String,
    license_id: String,
    license_name: String,
    open_issues_count: u32,
    pushed_at: String,
    date: String,
    stargazers_count: u32,
}

#[tokio::main]
async fn main() {
    let matches = clap_app!(myapp =>
        (version: VERSION.unwrap())
        (author: "Emil Sayahi <limesayahi@gmail.com>")
        (@arg INPUT: "The working directory of the grabber")
    )
    .get_matches();
    env::set_current_dir(matches.value_of("INPUT").unwrap()).unwrap();
    let octocrab;
    match fs::read_to_string("./token.txt") {
        Ok(_) => {
            octocrab = octocrab::OctocrabBuilder::new()
                .personal_token(fs::read_to_string("./token.txt").unwrap())
                .build()
                .unwrap();
        }
        Err(_) => {
            octocrab = octocrab::OctocrabBuilder::new().build().unwrap();
        }
    }

    let organizations = ["Dirout"];
    let mut repositories = vec![];
    for organization in &organizations {
        let organization = octocrab.orgs(organization.to_owned());
        let listing = organization
            .list_repos()
            .sort(params::repos::Sort::Pushed)
            .per_page(100)
            .send()
            .await
            .unwrap();
        for repository in listing.items {
            let license_id;
            let license_name;
            match &repository.license {
                Some(license) => {
                    license_id = license.clone().spdx_id.to_lowercase();
                    license_name = license.clone().name;
                }
                None => {
                    license_id = String::new();
                    license_name = String::new();
                }
            }

            let description;
            match repository.description {
                Some(gh_description) => {
                    description = gh_description;
                }
                None => {
                    description = String::new();
                }
            }

            let open_issues_count;
            match repository.has_issues.unwrap() {
                true => {
                    open_issues_count = repository.open_issues_count.unwrap();
                }
                false => {
                    open_issues_count = 0;
                }
            }

            let structured_data = Repository {
                archived: repository.archived.unwrap(),
                forked: repository.fork,
                title: repository.name,
                description: description.to_owned(),
                has_issues: repository.has_issues.unwrap(),
                has_license: repository.license.is_some(),
                html_url: repository.html_url.to_string(),
                license_id: license_id.to_owned(),
                license_name: license_name.to_owned(),
                open_issues_count,
                pushed_at: repository.pushed_at.unwrap().to_rfc3339(),
                date: repository.pushed_at.unwrap().to_rfc3339(),
                stargazers_count: repository.stargazers_count.unwrap(),
            };
            repositories.push(structured_data);
        }
    }

    for i in 0..repositories.len() {
        let file_contents = &serde_yaml::to_string(&repositories[i]).unwrap();
        write_file(
            format!("./repos/{}.mokkf", i),
            format!(
                "{}\ncollection:  \"repos\"\npermalink: date\n---",
                file_contents
            ),
        )
    }
}

fn write_file(path: String, text_to_write: String) {
    fs::create_dir_all(Path::new(&path[..]).parent().unwrap()).unwrap(); // Create output path, write to file
    let file = File::create(&path).unwrap(); // Create file which we will write to
    let mut buffered_writer = BufWriter::new(file); // Create a buffered writer, allowing us to modify the file we've just created
    write!(buffered_writer, "{}", text_to_write).unwrap(); // Write String to file
    buffered_writer.flush().unwrap(); // Empty out the data in memory after we've written to the file
}
