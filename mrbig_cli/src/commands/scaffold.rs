// ############################################################################
// #                                                                          #
// # mrbig_cli/src/commands/scaffold.rs                                       #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Command for generating seminal projects from templates.     #
// ############################################################################

//! Project scaffold command.
//!
//! # Examples
//!
//! ```ignore
//! > mrbig new service --from-template basics/multi-services --with-grpc
//! ```

// Import external dependencies
use console::style;
use heck::{KebabCase, SnakeCase};
use std::env;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

// Import internal dependencies
use crate::errors::Result;
use crate::errors::ScaffoldError as Error;
use crate::utilities::emojis;

/// Scaffold command options.
#[derive(Debug, StructOpt)]
pub struct ScaffoldCommandOptions {
    #[structopt(long = "--from-template", short = "t")]
    pub from_template: Option<String>,

    #[structopt(long = "--with-grpc", short = "g")]
    pub with_grpc: bool,

    #[structopt(long = "--name", short = "n")]
    pub name: Option<String>,

    #[structopt(parse(from_os_str), long = "--output-path", short = "o")]
    pub output_path: Option<PathBuf>,
    // TODO (mab) - Add more options
}

/// Default project scaffold options builder
impl Default for ScaffoldCommandOptions {
    fn default() -> Self {
        Self {
            from_template: None,
            with_grpc: true,
            name: None,
            output_path: None,
        }
    }
}

/// Executes the scaffolding command, passing arguments given at command-line.
pub fn execute(options: ScaffoldCommandOptions) -> Result<()> {
    debug!("Execute scaffold command with options: {:?}", options);

    let output_path = options.output_path.expect("missing --output-path option");

    let name = options.name.unwrap_or_else(|| {
        output_path
            .as_path()
            .file_name()
            .and_then(|f| f.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unnamed".into())
    });

    let from_tpl: PathBuf = options
        .from_template
        .expect("missing --from-template url")
        .into();

    // Create new project dir but fail if it already exists.
    let dir_path = create_project_dir(output_path.to_str().unwrap())?;

    // Copy all files from the template dir into the new project dir.
    // Template engine works over all files afterwards.
    copy_all(&dir_path, &from_tpl)?;

    // Create a template object
    let mut template = liquid::value::Object::new();
    template.insert(
        "project-name".into(),
        liquid::value::Value::scalar(name.clone()),
    );
    template.insert(
        "crate_name".into(),
        liquid::value::Value::scalar(name.to_snake_case()),
    );

    // Run template engine
    run_template_engine(template, dir_path)?;

    Ok(())
}

fn run_template_engine<P: AsRef<Path>>(template: liquid::value::Object, dir_path: P) -> Result<()> {
    let parser = liquid::ParserBuilder::new()
        .build()
        .expect("failed to build parser");

    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let new_contents = parser.clone().parse_file(&path)?.render(&template)?;
        std::fs::write(path, new_contents)?;
    }

    Ok(())
}

fn copy_all<P: AsRef<Path>>(dst: P, src: P) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();

        // ignore ".git" folder
        if path.as_path().ends_with(".git") {
            continue;
        }

        let file_dest: PathBuf = path.file_name().expect("file without name").into();
        let obj_dest: PathBuf = dst.as_ref().join(file_dest.clone());

        if path.is_dir() {
            std::fs::create_dir(&obj_dest)?;
            copy_all(obj_dest, path)?;
        } else {
            std::fs::File::create(&obj_dest)?;
            std::fs::copy(path, obj_dest)?;
        }
    }

    Ok(())
}

fn create_project_dir(name: &str) -> Result<PathBuf> {
    let dir_name = name.to_kebab_case();
    let project_dir = env::current_dir()
        .unwrap_or_else(|_e| ".".into())
        .join(&dir_name);

    println!(
        "{} {} `{}`{}",
        emojis::WRENCH,
        style("Creating project called").bold(),
        style(&dir_name).bold().yellow(),
        style("...").bold()
    );

    if project_dir.exists() {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!(
                "directory {} already exists",
                project_dir.to_str().unwrap_or("none")
            )
            .as_str(),
        )))
    } else {
        std::fs::create_dir(&project_dir)?;
        Ok(project_dir)
    }
}
