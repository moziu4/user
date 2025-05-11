use std::fs;
use std::sync::Arc;
use crate::core::domain::user::user_error::UserError;
use crate::core::domain::user::user_type::{NewUser, UserData};
use crate::core::operation::user_ops::UserOps;

pub async fn run_intern_user_command<'a>(
    user_ops: UserOps<'a>,
    json_path: &str,
) where
    'a: 'static,
{
    let json_data = match fs::read_to_string(json_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error al leer el archivo JSON: {}", e);
            return;
        }
    };

    // Parsear el JSON a `UserData`
    let user_data: UserData = match serde_json::from_str(&json_data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error al parsear el JSON: {}", e);
            return;
        }
    };

    // Crear el objeto `NewUser`
    let user_new = NewUser {
        username: user_data.username,
        email: user_data.email,
        password: user_data.password,
        name: user_data.name,
    };

    // Llamar a `intern_new_user` en `UserOps`
    match user_ops.create_user(user_new, true).await {
        Ok(user) => {
            println!("Usuario interno creado exitosamente: {:?}", user);
        }
        Err(UserError::EmailIsUsed) => {
            eprintln!("Error: El correo ya está en uso.");
        }
        Err(UserError::IncorrectFormatEmail) => {
            eprintln!("Error: El correo tiene un formato incorrecto.");
        }
        Err(UserError::InvalidUserId) => {
            eprintln!("Error: ID de usuario inválido.");
        }
        Err(UserError::HashPasswordError) => {
            eprintln!("Error: No se pudo generar el hash de la contraseña.");
        }
        Err(UserError::PermError) => {
            eprintln!("Error: Problema al cargar permisos.");
        }
        Err(UserError::AuthError) => {
            eprintln!("Error: Problema al crear la autenticación.");
        }
        Err(e) => {
            eprintln!("Error inesperado: {:?}", e);
        }
    }
}
