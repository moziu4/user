use clap::Parser;

pub mod user_command;
pub mod permsrelationship_commands;


#[derive(Parser, Debug)]
#[command(name = "User Management", about = "Comandos para gestionar usuarios")]
pub enum Commands {
    InternNewUser,
    CreatePermsRelationship,
}