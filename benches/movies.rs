//! # Movies
use std::{
    fs::File,
    hint::black_box,
    path::{Path, PathBuf},
    sync::atomic::{AtomicI32, Ordering},
};

use anyhow::Result;
use criterion::async_executor::FuturesExecutor;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use geekorm::{ConnectionManager, prelude::*};

static INSERT_MOVIE_ID: AtomicI32 = AtomicI32::new(9_000_000);

#[derive(Table, Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Movies {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<u64>,

    #[geekorm(unique)]
    pub movie_id: i32,

    #[geekorm(search)]
    pub title: String,

    pub cast_json: String,
    pub crew: String,
}

#[derive(Debug, serde::Deserialize)]
struct MoviesDataset {
    pub movie_id: i32,
    pub title: String,
    #[serde(rename = "cast")]
    pub cast_json: String,
    pub crew: String,
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap_or_else(|error| {
        panic!("failed to create benchmark runtime: {error}");
    });

    let database_path = std::env::temp_dir().join("geekorm-movies-credits-v2.sqlite");
    let insert_database_path = std::env::temp_dir().join("geekorm-movies-insert-v2.sqlite");
    let _ = std::fs::remove_file(&insert_database_path);
    runtime.block_on(async {
        setup(&database_path).await.unwrap_or_else(|error| {
            panic!("failed to setup movie benchmark database: {error}");
        });
    });

    let mut group = c.benchmark_group("movies");
    group.sample_size(10);

    group.bench_with_input(BenchmarkId::new("setup_load", 5000), &5000, |b, _| {
        b.to_async(FuturesExecutor).iter(|| async {
            let setup_path = std::env::temp_dir().join("geekorm-movies-setup.sqlite");
            let _ = std::fs::remove_file(&setup_path);
            let database = setup(&setup_path).await.unwrap_or_else(|error| {
                panic!("failed to setup movie benchmark database: {error}");
            });
            let count = Movies::total(&database.acquire().await)
                .await
                .unwrap_or_else(|error| panic!("failed to count movies: {error}"));
            black_box(count);
        });
    });

    group.bench_function("total", |b| {
        let database_path = database_path.clone();
        b.to_async(FuturesExecutor).iter(|| async {
            let database = setup(&database_path).await.unwrap_or_else(|error| {
                panic!("failed to setup movie benchmark database: {error}");
            });
            let count = Movies::total(&database.acquire().await)
                .await
                .unwrap_or_else(|error| panic!("failed to count movies: {error}"));
            black_box(count);
        });
    });

    group.bench_function("fetch_by_movie_id", |b| {
        let database_path = database_path.clone();
        b.to_async(FuturesExecutor).iter(|| async {
            let database = setup(&database_path).await.unwrap_or_else(|error| {
                panic!("failed to setup movie benchmark database: {error}");
            });
            let movie = Movies::fetch_by_movie_id(&database.acquire().await, 19995)
                .await
                .unwrap_or_else(|error| panic!("failed to fetch movie by id: {error}"));
            black_box(movie);
        });
    });

    group.bench_function("all", |b| {
        let database_path = database_path.clone();
        b.to_async(FuturesExecutor).iter(|| async {
            let database = setup(&database_path).await.unwrap_or_else(|error| {
                panic!("failed to setup movie benchmark database: {error}");
            });
            let movies = Movies::all(&database.acquire().await)
                .await
                .unwrap_or_else(|error| panic!("failed to fetch all movies: {error}"));
            black_box(movies);
        });
    });

    group.bench_function("search_title", |b| {
        let database_path = database_path.clone();
        b.to_async(FuturesExecutor).iter(|| async {
            let database = setup(&database_path).await.unwrap_or_else(|error| {
                panic!("failed to setup movie benchmark database: {error}");
            });
            let movies = Movies::search(&database.acquire().await, "dark")
                .await
                .unwrap_or_else(|error| panic!("failed to search movies: {error}"));
            black_box(movies);
        });
    });

    group.bench_function("insert", |b| {
        let database_path = insert_database_path.clone();
        b.to_async(FuturesExecutor).iter(|| async {
            let database = setup(&database_path).await.unwrap_or_else(|error| {
                panic!("failed to setup movie benchmark database: {error}");
            });
            let movie_id = INSERT_MOVIE_ID.fetch_add(1, Ordering::Relaxed);
            let mut movie = Movies::new(
                movie_id,
                "Criterion Movie".to_string(),
                "[]".to_string(),
                "[]".to_string(),
            );
            movie
                .save(&database.acquire().await)
                .await
                .unwrap_or_else(|error| panic!("failed to insert movie: {error}"));
            black_box(movie);
        });
    });

    group.finish();
}

async fn setup(path: &Path) -> Result<ConnectionManager> {
    let database = ConnectionManager::path(path).await?;
    Movies::create_table(&database.acquire().await).await?;

    let movie_count = Movies::total(&database.acquire().await).await?;

    if movie_count == 0 {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("benches")
            .join("data")
            .join("tmdb_5000_credits.csv");
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for movie_data in rdr.deserialize() {
            let movie: MoviesDataset = movie_data?;

            let mut movie_record =
                Movies::new(movie.movie_id, movie.title, movie.cast_json, movie.crew);
            movie_record.save(&database.acquire().await).await?;
        }
    }

    Ok(database)
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
