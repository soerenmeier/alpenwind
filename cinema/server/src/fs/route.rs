//! Serve movies, posters and series

use crate::CinemaConf;

use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use std::sync::OnceLock;

use chuchi::error::{ClientErrorKind, ServerErrorKind};
use chuchi::extractor::PathStr;
use chuchi::fs::{serve_file, Caching, IntoPathBuf};
use chuchi::{get, Chuchi, Error, Request, Response, Result};

use core_lib::users::{CheckedUser, DataToken, RightsAny};

use tokio::fs;

use image::error::ImageError;
use image::imageops::FilterType;

static CACHING: OnceLock<Caching> = OnceLock::new();

fn get_caching() -> Caching {
	CACHING.get_or_init(|| Caching::default()).clone()
}

#[get("/assets/cinema/movies/{*path}")]
async fn get_movies(
	req: &mut Request,
	path: &PathStr,
	_sess: CheckedUser<RightsAny, DataToken>,
	cfg: &CinemaConf,
) -> Result<Response> {
	let req_path = into_path_buf(&path)?;
	let file_path = Path::new(&cfg.movies_dir).join(req_path);

	serve_file(file_path, &req, Some(get_caching()))
		.await
		.map_err(Error::from_client_io)
}

// movie posters
#[get("/assets/cinema/posters/movies/{*path}")]
async fn get_posters_movies(
	req: &mut Request,
	path: &PathStr,
	cfg: &CinemaConf,
) -> Result<Response> {
	let req_path = into_path_buf(&path)?;

	// get the path of the poster
	let scaled_path = Path::new(&cfg.scaled_movies_posters).join(&req_path);
	let full_path = Path::new(&cfg.movie_posters_dir).join(&req_path);

	// scale image if it is missing
	scale_if_missing(scaled_path.clone(), full_path).await?;

	serve_file(scaled_path, &req, Some(get_caching()))
		.await
		.map_err(Error::from_client_io)
}

// full movie posters
#[get("/assets/cinema/full-posters/movies/{*path}")]
async fn get_full_posters_movies(
	req: &mut Request,
	path: &PathStr,
	cfg: &CinemaConf,
) -> Result<Response> {
	let req_path = into_path_buf(&path)?;
	let file_path = Path::new(&cfg.movie_posters_dir).join(req_path);

	serve_file(file_path, &req, Some(get_caching()))
		.await
		.map_err(Error::from_client_io)
}

// series
#[get("/assets/cinema/series/{*path}")]
async fn get_series(
	req: &mut Request,
	path: &PathStr,
	_sess: CheckedUser<RightsAny, DataToken>,
	cfg: &CinemaConf,
) -> Result<Response> {
	let req_path = into_path_buf(&path)?;
	let file_path = Path::new(&cfg.series_dir).join(req_path);

	serve_file(file_path, &req, Some(get_caching()))
		.await
		.map_err(Error::from_client_io)
}

// series posters
#[get("/assets/cinema/posters/series/{*path}")]
async fn get_posters_series(
	req: &mut Request,
	path: &PathStr,
	cfg: &CinemaConf,
) -> Result<Response> {
	let req_path = into_path_buf(&path)?;

	// get the path of the poster
	let scaled_path = Path::new(&cfg.scaled_series_posters).join(&req_path);
	let mut full_path = Path::new(&cfg.series_dir).join(&req_path);
	series_poster_transform(&mut full_path)?;

	// scale image if it is missing
	scale_if_missing(scaled_path.clone(), full_path).await?;

	serve_file(scaled_path, &req, Some(get_caching()))
		.await
		.map_err(Error::from_client_io)
}

// full series posters
#[get("/assets/cinema/full-posters/series/{*path}")]
async fn get_full_posters_series(
	req: &mut Request,
	path: &PathStr,
	cfg: &CinemaConf,
) -> Result<Response> {
	// we need to transform series.jpg to series/poster.jpg
	let req_path = into_path_buf(&path)?;
	let mut file_path = Path::new(&cfg.series_dir).join(req_path);
	series_poster_transform(&mut file_path)?;

	serve_file(file_path, &req, Some(get_caching()))
		.await
		.map_err(Error::from_client_io)
}

fn into_path_buf(uri: &str) -> Result<PathBuf> {
	uri.into_path_buf()
		.map_err(|e| Error::new(ClientErrorKind::BadRequest, e))
}

// transform <series>.jpg to <series>/poster.jpg
fn series_poster_transform(path: &mut PathBuf) -> Result<()> {
	if !matches!(path.extension(), Some(ext) if ext == "jpg") {
		return Err(Error::new(ClientErrorKind::BadRequest, "expected a jpg"));
	}

	path.set_extension("");
	path.push("poster.jpg");

	Ok(())
}

async fn scale_if_missing(
	scaled_path: PathBuf,
	full_path: PathBuf,
) -> Result<()> {
	let exists = fs::metadata(&scaled_path)
		.await
		.map(|m| m.is_file())
		.unwrap_or(false);
	if exists {
		return Ok(());
	}

	let exists = fs::metadata(&full_path)
		.await
		.map(|m| m.is_file())
		.unwrap_or(false);
	if !exists {
		eprintln!("full path not exists {full_path:?}");
		return Err(Error::new(ClientErrorKind::NotFound, ""));
	}

	// todo should this be locked??
	tokio::task::spawn_blocking(move || scale_sync(full_path, scaled_path))
		.await
		.unwrap()
		.map_err(|e| Error::new(ServerErrorKind::InternalServerError, e))
}

fn scale_sync(
	full_path: PathBuf,
	scaled_path: PathBuf,
) -> StdResult<(), ImageError> {
	let image = image::io::Reader::open(full_path)
		.map_err(ImageError::IoError)?
		.decode()?;

	let aspect_radio = image.width() as f32 / image.height() as f32;
	let n_width: u32 = 300;
	let n_height = (n_width as f32 / aspect_radio) as u32;

	let image = image.resize_exact(n_width, n_height, FilterType::Lanczos3);
	image.save(scaled_path)
}

pub(crate) fn add_routes(server: &mut Chuchi) {
	server.add_route(get_movies);
	server.add_route(get_posters_movies);
	server.add_route(get_full_posters_movies);
	server.add_route(get_series);
	server.add_route(get_posters_series);
	server.add_route(get_full_posters_series);
}
