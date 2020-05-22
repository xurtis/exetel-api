use anyhow::Error;
use structopt::StructOpt;
use dialoguer::Password;
use exetel_api::{Authorization, customer};

/// Command line utility to query the Exetel web API
#[derive(StructOpt)]
struct Args {
    /// Username to authenticate to API
    #[structopt(short, long)]
    username: Option<String>,
    /// Password (prompted interactively)
    #[structopt(skip)]
    password: Option<String>,
    /// Authoirzation token
    #[structopt(skip)]
    authorization: Option<Authorization>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut args = Args::from_args();

    if args.username.is_some() {
        let password = Password::new().with_prompt("Enter password").interact()?;
        args.password = Some(password);
    }

    if let (Some(username), Some(password)) = (args.username, args.password) {
        let authorization = Authorization::authenticate(&username, &password).await?;
        let client = authorization.into_client()?;
        println!("Services: {:#?}", client.services().await?);
    }

    Ok(())
}
