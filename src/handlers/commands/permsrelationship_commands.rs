use std::fs;
use crate::core::domain::perm::perm_type::PermsRelationship;
use crate::core::operation::perms_ops::{PermOps};

pub async fn run_create_perms_relationship_command<'a>(
    perm_ops: &'a PermOps<'a>, 
    json_path: &str,
)
{
    // Leer el archivo JSON
    let json_data = match fs::read_to_string(json_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error al leer el archivo JSON: {}", e);
            return;
        }
    };

    // Parsear el JSON a un vector de `PermsRelationship`
    let perms_relationships: Vec<PermsRelationship> = match serde_json::from_str(&json_data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error al parsear el JSON: {}", e);
            return;
        }
    };

    // Llamar al método `create_perms_relationship` en `PermOps`
    match perm_ops.create_perms_relationship(perms_relationships).await {
        Ok(()) => {
            println!("Se crearon correctamente las relaciones de permisos.");
        }
        Err(e) => {
            eprintln!("Error al crear relaciones de permisos: {:?}", e);
        }
    }
}
