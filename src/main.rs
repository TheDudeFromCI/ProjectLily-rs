use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use log::{error, info};
use project_lily::agent::{Agent, AgentSettings};
use project_lily::communications::discord;
use project_lily::llm::llama_cpp::LlamaCppServer;
use project_lily::llm::LlmWrapper;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    agent: PathBuf,
}

#[tokio::main]
async fn main() -> ExitCode {
    pretty_env_logger::formatted_timed_builder()
        .parse_default_env()
        .filter_module("hyper", log::LevelFilter::Warn)
        .filter_module("tracing", log::LevelFilter::Warn)
        .filter_module("serenity", log::LevelFilter::Warn)
        .filter_module("rustls", log::LevelFilter::Warn)
        .filter_module("h2", log::LevelFilter::Warn)
        .filter_module("reqwest", log::LevelFilter::Warn)
        .filter_module("cached_path", log::LevelFilter::Warn)
        .filter_module("tungstenite", log::LevelFilter::Warn)
        .init();

    let args = Args::parse();

    info!("Loading Agent File: {}", args.agent.display());
    if !args.agent.exists() {
        error!("Agent file does not exist: {}", args.agent.display());
        return ExitCode::FAILURE;
    }

    let agent_settings = match AgentSettings::from_file(&args.agent) {
        Ok(settings) => settings,
        Err(err) => {
            error!("{}", err);
            return ExitCode::FAILURE;
        }
    };

    info!("Connecting to LLM Server");
    let llm: LlmWrapper = LlamaCppServer::default().into();
    match llm.validate_connection().await {
        Ok(_) => {}
        Err(err) => {
            error!("{}", err);
            return ExitCode::FAILURE;
        }
    }

    info!("Creating Agent instance");
    let mut agent = match Agent::new(agent_settings, llm).await {
        Ok(agent) => agent,
        Err(err) => {
            error!("{}", err);
            return ExitCode::FAILURE;
        }
    };

    info!("Connecting to Discord");
    discord::run(&mut agent.communication_manager);

    loop {
        if let Err(err) = agent.update().await {
            error!("{}", err);
            return ExitCode::FAILURE;
        }
    }
}
