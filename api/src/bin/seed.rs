//! Dev-only data seeder.
//!
//! Wipes artifacts/artists/tribes (in FK order) and reinserts a curated set
//! of well-known Aboriginal artists, their tribes, and notable works. Run:
//!
//!     cargo run --bin seed
//!
//! Idempotent — every run gives you the same end state. Destroys any
//! existing rows in those three tables.

use anyhow::Context;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL must be set (see .env.example)")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Wiping artifacts, artists, tribes...");
    sqlx::query!("TRUNCATE artifacts, artists, tribes RESTART IDENTITY")
        .execute(&pool)
        .await?;

    println!("Seeding tribes...");
    let tribes = seed_tribes(&pool).await?;

    println!("Seeding artists...");
    let artists = seed_artists(&pool, &tribes).await?;

    println!("Seeding artifacts...");
    seed_artifacts(&pool, &artists).await?;

    println!(
        "Done. {} tribes, {} artists, 7 artifacts.",
        tribes.len(),
        artists.len()
    );
    Ok(())
}

#[derive(Clone)]
struct Seeded {
    id: Uuid,
    name: &'static str,
}

async fn seed_tribes(pool: &PgPool) -> anyhow::Result<Vec<Seeded>> {
    let inputs = [
        (
            "Pintupi",
            "Western Desert",
            "Western Desert language",
            "Aboriginal Australian people from the Gibson Desert region of Western Australia and the Northern Territory.",
        ),
        (
            "Yolngu",
            "Arnhem Land",
            "Yolngu Matha",
            "Aggregation of Aboriginal Australian peoples inhabiting north-eastern Arnhem Land.",
        ),
        (
            "Anmatyerre",
            "Central Desert",
            "Anmatyerr (Arandic)",
            "Aboriginal Australian people of the Central Desert region, north of Mparntwe (Alice Springs).",
        ),
        (
            "Arrernte",
            "Central Australia",
            "Arrernte (Arandic)",
            "Group of Aboriginal Australian peoples whose traditional lands include Mparntwe (Alice Springs).",
        ),
        (
            "Tiwi",
            "Tiwi Islands",
            "Tiwi (language isolate)",
            "Aboriginal Australian people of the Tiwi Islands, north of Darwin in the Timor Sea.",
        ),
    ];

    let mut out = Vec::with_capacity(inputs.len());
    for (name, region, language_group, description) in inputs {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO tribes (name, region, language_group, description)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            name,
            region,
            language_group,
            description,
        )
        .fetch_one(pool)
        .await?;
        out.push(Seeded { id, name });
        println!("  + {name}");
    }
    Ok(out)
}

async fn seed_artists(pool: &PgPool, tribes: &[Seeded]) -> anyhow::Result<Vec<Seeded>> {
    let inputs: [(
        &'static str,
        Option<i16>,
        Option<i16>,
        Option<&str>,
        Option<&str>,
        Option<&str>,
    ); 8] = [
        (
            "Emily Kame Kngwarreye",
            Some(1910),
            Some(1996),
            Some("Utopia, Northern Territory"),
            Some("One of Australia's most prominent contemporary Aboriginal artists, known for vibrant, gestural paintings of her Country."),
            Some("Anmatyerre"),
        ),
        (
            "Albert Namatjira",
            Some(1902),
            Some(1959),
            Some("Hermannsburg, Northern Territory"),
            Some("Watercolour painter of Central Australian landscapes; often credited as the first internationally celebrated Aboriginal artist."),
            Some("Arrernte"),
        ),
        (
            "Rover Thomas",
            Some(1926),
            Some(1998),
            Some("Kimberley region, Western Australia"),
            Some("Pioneering East Kimberley painter; co-founder of the Warmun art movement."),
            None,
        ),
        (
            "Clifford Possum Tjapaltjarri",
            Some(1932),
            Some(2002),
            Some("Napperby Station, Northern Territory"),
            Some("A founding member of the Western Desert art movement; known for large-scale topographical 'map' paintings."),
            Some("Anmatyerre"),
        ),
        (
            "Mick Namarari Tjapaltjarri",
            Some(1926),
            Some(1998),
            Some("Marnpi, Northern Territory"),
            Some("Founding member of the Papunya Tula movement; later associated with the Pintupi return-to-Country period."),
            Some("Pintupi"),
        ),
        (
            "Naata Nungurrayi",
            Some(1932),
            Some(2021),
            Some("Kintore, Northern Territory"),
            Some("Senior Pintupi painter; works often depict women's ceremony at sacred sites near her Country."),
            Some("Pintupi"),
        ),
        (
            "Gulumbu Yunupingu",
            Some(1945),
            Some(2012),
            Some("Gunyangara, Arnhem Land"),
            Some("Yolngu artist whose paintings of Garak (the universe) gained international recognition."),
            Some("Yolngu"),
        ),
        (
            "Kitty Kantilla",
            Some(1928),
            Some(2003),
            Some("Milikapiti, Tiwi Islands"),
            Some("Senior Tiwi artist who worked in painting, printmaking and sculpture, often depicting jilamara designs."),
            Some("Tiwi"),
        ),
    ];

    let mut out = Vec::with_capacity(inputs.len());
    for (name, birth, death, region, biography, tribe_name) in inputs {
        let tribe_id = tribe_name.and_then(|n| find_id(tribes, n));
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO artists (display_name, birth_year, death_year, region, biography, tribe_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            name,
            birth,
            death,
            region,
            biography,
            tribe_id,
        )
        .fetch_one(pool)
        .await?;
        out.push(Seeded { id, name });
        println!("  + {name}");
    }
    Ok(out)
}

async fn seed_artifacts(pool: &PgPool, artists: &[Seeded]) -> anyhow::Result<()> {
    let inputs: [(
        &str,
        &str,
        Option<&str>,
        Option<&str>,
        Option<&str>,
        Option<i16>,
        Option<i16>,
        Option<i16>,
        Option<&str>,
    ); 7] = [
        (
            "Earth's Creation",
            "Emily Kame Kngwarreye",
            Some("painting"),
            Some("Western Desert"),
            Some("Synthetic polymer paint on canvas, four panels"),
            Some(1994),
            Some(275),
            Some(632),
            Some("A monumental four-panel painting referencing the abundance of her Country during the wet season."),
        ),
        (
            "Hermannsburg Landscape",
            "Albert Namatjira",
            Some("painting"),
            Some("Hermannsburg School"),
            Some("Watercolour on paper"),
            Some(1950),
            Some(38),
            Some(56),
            Some("A characteristic Central Australian landscape with white-trunked ghost gums against red ranges."),
        ),
        (
            "Roads Crossing the Salt Pan",
            "Rover Thomas",
            Some("painting"),
            Some("East Kimberley"),
            Some("Natural earth pigments and bush gum on canvas"),
            Some(1986),
            Some(90),
            Some(180),
            Some("Recounts country traversed during a Dreaming journey, with broad ochre fields and white dotted lines marking the route."),
        ),
        (
            "Warlugulong",
            "Clifford Possum Tjapaltjarri",
            Some("painting"),
            Some("Western Desert"),
            Some("Synthetic polymer paint on canvas"),
            Some(1977),
            Some(202),
            Some(338),
            Some("A topographical narrative depicting the ancestral fire Dreaming and related stories spanning the artist's Country."),
        ),
        (
            "Marrapinti",
            "Naata Nungurrayi",
            Some("painting"),
            Some("Western Desert"),
            Some("Synthetic polymer paint on linen"),
            Some(2002),
            Some(122),
            Some(153),
            Some("Depicts the rockhole site of Marrapinti, west of Pollock Hills, where ancestral women gathered."),
        ),
        (
            "Garak — The Universe",
            "Gulumbu Yunupingu",
            Some("painting"),
            Some("Yirrkala bark"),
            Some("Natural pigments on bark"),
            Some(2004),
            Some(155),
            Some(60),
            Some("A bark painting representing the vastness of Garak — the universe — and the artist's place within it."),
        ),
        (
            "Jilamara",
            "Kitty Kantilla",
            Some("painting"),
            Some("Tiwi"),
            Some("Natural ochres on canvas"),
            Some(2000),
            Some(120),
            Some(90),
            Some("Tiwi body-painting designs translated to canvas, with bold geometric patterning in ochre, white and black."),
        ),
    ];

    for (title, artist_name, art_type, art_style, medium, year, height, width, description) in
        inputs
    {
        let artist_id = find_id(artists, artist_name)
            .unwrap_or_else(|| panic!("seed: missing artist {artist_name}"));
        sqlx::query!(
            r#"
            INSERT INTO artifacts (title, artist_id, art_type, art_style, medium,
                                   year_created, height_cm, width_cm, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            title,
            artist_id,
            art_type,
            art_style,
            medium,
            year,
            height,
            width,
            description,
        )
        .execute(pool)
        .await?;
        println!("  + {title}");
    }
    Ok(())
}

fn find_id(rows: &[Seeded], name: &str) -> Option<Uuid> {
    rows.iter().find(|r| r.name == name).map(|r| r.id)
}
