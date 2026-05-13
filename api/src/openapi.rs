//! OpenAPI 3 description for the whole API, plus the Swagger UI mounting.
//!
//! Source of truth for the description lives on the handler `#[utoipa::path]`
//! attributes and the `#[derive(ToSchema)]` impls on the model structs. This
//! module just stitches them into a single document and registers the JWT
//! security scheme that protected endpoints reference.

use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use crate::{
    artifacts::{
        model::{Artifact, ArtifactInput},
        routes as artifact_routes,
    },
    artists::{
        model::{Artist, ArtistInput},
        routes as artist_routes,
    },
    auth::Role,
    tribes::{
        model::{Tribe, TribeInput},
        routes as tribe_routes,
    },
    users::{
        model::{AuthResponse, LoginInput, RegisterInput, User, UserUpdate},
        routes as user_routes,
    },
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Aboriginal Art Gallery API",
        version = "0.1.0",
        description = "Backend service for the Aboriginal Art Gallery of Australia. \
            Exposes the Artists, Artifacts, and Tribes bounded contexts plus a Users + JWT auth layer. \
            Reads are public; writes require an Admin-role JWT in the `Authorization` header.",
        contact(name = "Michael Pappas")
    ),
    paths(
        // Artists
        artist_routes::list_artists,
        artist_routes::get_artist,
        artist_routes::create_artist,
        artist_routes::update_artist,
        artist_routes::delete_artist,

        // Artifacts
        artifact_routes::list_artifacts,
        artifact_routes::get_artifact,
        artifact_routes::create_artifact,
        artifact_routes::update_artifact,
        artifact_routes::delete_artifact,

        // Tribes
        tribe_routes::list_tribes,
        tribe_routes::get_tribe,
        tribe_routes::create_tribe,
        tribe_routes::update_tribe,
        tribe_routes::delete_tribe,

        // Auth + users
        user_routes::register,
        user_routes::login,
        user_routes::me,
        user_routes::list_users,
        user_routes::get_user,
        user_routes::update_user,
        user_routes::delete_user,
    ),
    components(schemas(
        Artist, ArtistInput,
        Artifact, ArtifactInput,
        Tribe, TribeInput,
        User, RegisterInput, LoginInput, AuthResponse, UserUpdate, Role,
    )),
    tags(
        (name = "artists",   description = "Individual makers — biographies, era, tribal affiliation"),
        (name = "artifacts", description = "Objects, paintings, and ceremonial pieces"),
        (name = "tribes",    description = "Aboriginal peoples, language groups, and Country"),
        (name = "auth",      description = "Register, login, and the current-user endpoint"),
        (name = "users",     description = "Admin-only user management"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Registers the HTTP `Bearer <JWT>` security scheme under the name
/// `bearer_auth`. Individual `#[utoipa::path]` attributes reference this name
/// via `security(("bearer_auth" = []))` to mark protected endpoints.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi
            .components
            .as_mut()
            .expect("components are auto-created by the OpenApi derive");
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
