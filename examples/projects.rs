mod projects {
    pub mod args {
        use cling::prelude::*;

        #[derive(Run, Parser, Debug, Clone)]
        #[command(author, version, about, long_about = None)]
        #[cling(run = "super::handlers::init")]
        pub struct AppArgs {
            #[clap(flatten)]
            #[cling(collect)]
            pub common: CommonArgs,
            #[command(subcommand)]
            pub cmd: Commands,
        }

        #[derive(Run, Subcommand, Debug, Clone)]
        pub enum Commands {
            /// Creates a new project
            CreateProject(CreateProjectArgs),
            /// Lists existing projects
            ListProjects(ListProjectArgs),
            /// Print the system version
            #[cling(run = "super::handlers::print_version")]
            PrintVersion,
        }

        #[derive(Args, Collect, Debug, Clone)]
        pub struct CommonArgs {
            /// Turn debugging information on
            #[arg(short, long, action = clap::ArgAction::Count, global = true)]
            pub debug: u8,
            /// User access token
            #[arg(long, global = true)]
            pub access_token: Option<String>,
        }

        #[derive(Run, Collect, Args, Debug, Clone)]
        #[cling(run = "super::handlers::create_project")]
        pub struct CreateProjectArgs {
            /// Name of the project
            pub name: String,
        }

        #[derive(Run, Collect, Args, Debug, Clone)]
        #[cling(run = "super::handlers::list_projects")]
        pub struct ListProjectArgs {
            /// Filter projects by name
            #[arg(short, long)]
            pub filter: Option<String>,
        }
    }

    // -- handlers --
    mod handlers {
        // cling_handler macro adds a few assertions to help you
        // investigate errors if the compiler is not happy about
        // your handler being attached to a #[cling(run = ...)].
        use anyhow::Result;
        use cling::{cling_handler, State};
        use log::debug;

        use super::args::*;

        /// Represents an authenticated user session.
        #[derive(Clone)]
        pub struct Session {
            user_id: String,
        }

        /// Initialize logging and create a session state object that will be
        /// passed to all downstream handlers.
        #[cling_handler]
        pub async fn init(common: &CommonArgs) -> Result<State<Session>> {
            if common.debug >= 1 {
                env_logger::builder()
                    .filter_level(log::LevelFilter::Debug)
                    .init();
            }
            // Fake, potentially calls a server in the wild.
            match common.access_token.as_ref() {
                | Some(token) if token == "very-secret-token" => {
                    Ok(State(Session {
                        user_id: "1234".to_string(),
                    }))
                }
                | Some(_) => Err(anyhow::anyhow!("Invalid access token")),
                | None => Err(anyhow::anyhow!("Access token required!")),
            }
        }

        // cling_handler macro adds a few assertions to help you investigate
        // errors if the compiler is not happy about your handler being
        // attached to a #[cling(run = ...)].
        #[cling_handler]
        pub async fn create_project(
            State(session): State<Session>,
            args: &CreateProjectArgs,
        ) -> Result<()> {
            println!(
                "Creating project '{}' for user {}.",
                args.name, session.user_id
            );
            debug!("Would have created the project here");
            Ok(())
        }

        #[cling_handler]
        pub async fn list_projects(
            State(session): State<Session>,
            args: &ListProjectArgs,
        ) -> Result<()> {
            println!(
                "Listing projects for user '{}' with filter={:?}.",
                session.user_id, args.filter
            );
            debug!("Would have listed all projects here");
            Ok(())
        }

        #[cling_handler]
        pub fn print_version() {
            println!("Version 0.1.0");
        }
    }
}

// -- main --
use cling::prelude::*;
use projects::args::AppArgs;

#[tokio::main]
async fn main() -> ClingFinished<AppArgs> {
    Cling::parse_and_run().await
}
