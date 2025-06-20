use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme}, Modify, OpenApi
};

struct DiscordAuthAddon;

impl Modify for DiscordAuthAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "user_jwt",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

struct UserApiKeyAddon;

impl Modify for UserApiKeyAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "user_api_key",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("API Key")
                        .build(),
                )
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&DiscordAuthAddon, &UserApiKeyAddon),
    servers(
        (url = "https://api.lumirad.io", description = "Live radio server"),
        (url = "http://localhost:{port}", description = "Local development server",
            variables(
                ("port" = (default = "8000", description = "Port number"))
            )
        )
    ),
    paths(
        crate::routes::auth::discord_login
    )
)]
pub struct ApiDoc;
