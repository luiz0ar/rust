pub mod schemas;
pub mod security;

use security::SecurityAddon;
use utoipa::OpenApi;

use crate::handlers::{auth_handler, user_handler};

/// Responsabilidade única: Orquestrador da especificação OpenAPI raiz,
/// combinando rotas, tags, segurança e schemas via composição.
#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handler::login,
        auth_handler::refresh,
        auth_handler::logout,
        auth_handler::profile,
        user_handler::create_user,
        user_handler::list_users,
        user_handler::get_user,
        user_handler::update_user,
        user_handler::delete_user,
    ),
    tags(
        (name = "Auth", description = "Operações de autenticação, renovação e perfil"),
        (name = "Users", description = "Gerenciamento e consulta de usuários")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

impl ApiDoc {
    /// Compõe a especificação final aplicando o merge dos submódulos
    pub fn build_spec() -> utoipa::openapi::OpenApi {
        let mut doc = Self::openapi();
        doc.merge(schemas::ApiSchemas::openapi());
        doc
    }
}
