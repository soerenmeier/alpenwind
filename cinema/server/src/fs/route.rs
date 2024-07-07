use crate::CinemaConf;

use std::os;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use std::sync::OnceLock;

use fire::error::{ClientErrorKind, ServerErrorKind};
use fire::extractor::{PathParam, PathStr};
use fire::fs::{serve_file, Caching, IntoPathBuf};
use fire::header::{Method, RequestHeader};
use fire::routes::Route;
use fire::util::PinnedFuture;
use fire::{get, Error, FireBuilder, Request, Response, Result};

use core_lib::users::Users;

use tokio::fs;

use image::error::ImageError;
use image::imageops::FilterType;

const MOVIES_URI: &str = "/assets/cinema/movies/";
const SERIES_DIR: &str = "/assets/cinema/series/";
const POSTERS_MOVIES_DIR: &str = "/assets/cinema/posters/movies/";
const POSTERS_SERIES_DIR: &str = "/assets/cinema/posters/series/";
const FULL_POSTERS_MOVIES_DIR: &str = "/assets/cinema/full-posters/movies/";
const FULL_POSTERS_SERIES_DIR: &str = "/assets/cinema/full-posters/series/";

#[derive(Debug, Clone)]
pub struct CinemaFsRoute {
	// /assets/cinema/movies/
	movies_dir: PathBuf,
	// /assets/cinema/posters/
	posters_dir: PathBuf,
	// /assets/cinema/series/
	series_dir: PathBuf,
	// /assets/cinema/posters/movies/
	scaled_movies_posters: PathBuf,
	// /assets/cinema/posters/series/
	scaled_series_posters: PathBuf,
	caching: Caching,
}

impl CinemaFsRoute {
	pub(crate) fn new(cfg: &CinemaConf) -> Self {
		Self {
			movies_dir: PathBuf::from(&cfg.movies_dir),
			posters_dir: PathBuf::from(&cfg.movie_posters_dir),
			series_dir: PathBuf::from(&cfg.series_dir),
			scaled_movies_posters: PathBuf::from(&cfg.scaled_movies_posters),
			scaled_series_posters: PathBuf::from(&cfg.scaled_series_posters),
			caching: Caching::default(),
		}
	}
}

static CACHING: OnceLock<Caching> = OnceLock::new();

fn get_caching() -> Caching {
	CACHING.get_or_init(|| Caching::default()).clone()
}

#[get("/assets/cinema/movies/{*path}")]
async fn get_movies(
	req: &mut Request,
	path: &PathStr,
	users: &Users,
	cfg: &CinemaConf,
) -> Result<Response> {
	let (_, _) = users
		.sess_user_from_cookie(req.header())
		.await
		.map_err(|e| Error::new(ClientErrorKind::BadRequest, e))?;

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
	users: &Users,
	cfg: &CinemaConf,
) -> Result<Response> {
	let (_, _) = users
		.sess_user_from_cookie(req.header())
		.await
		.map_err(|e| Error::new(ClientErrorKind::BadRequest, e))?;

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

// impl Route for CinemaFsRoute {
// 	fn check(&self, header: &RequestHeader) -> bool {
// 		let p = header.uri().path();

// 		header.method() == &Method::GET && p.starts_with(MOVIES_URI)
// 			|| p.starts_with(SERIES_DIR)
// 			|| p.starts_with(POSTERS_MOVIES_DIR)
// 			|| p.starts_with(POSTERS_SERIES_DIR)
// 			|| p.starts_with(FULL_POSTERS_MOVIES_DIR)
// 			|| p.starts_with(FULL_POSTERS_SERIES_DIR)
// 	}

// 	fn validate_data(&self, data: &Data) {
// 		assert!(data.exists::<Users>());
// 	}

// 	fn call<'a>(
// 		&'a self,
// 		req: &'a mut Request,
// 		data: &'a Data,
// 	) -> PinnedFuture<'a, Result<Response>> {
// 		PinnedFuture::new(async move {
// 			let users = data.get::<Users>().unwrap();

// 			// ignore jpegs
// 			if !req.header().uri().path().ends_with(".jpg") {
// 				let (_, _) = users
// 					.sess_user_from_cookie(req.header())
// 					.await
// 					.map_err(|e| Error::new(ClientErrorKind::BadRequest, e))?;
// 			}

// 			let p = req.header().uri().path();
// 			let file_path = if p.starts_with(MOVIES_URI) {
// 				let req_path = into_path_buf(&p[MOVIES_URI.len()..])?;
// 				self.movies_dir.join(req_path)
// 			} else if p.starts_with(SERIES_DIR) {
// 				let req_path = into_path_buf(&p[SERIES_DIR.len()..])?;
// 				self.series_dir.join(req_path)
// 			} else if p.starts_with(POSTERS_MOVIES_DIR) {
// 				// get the path of the poster
// 				let req_path = into_path_buf(&p[POSTERS_MOVIES_DIR.len()..])?;
// 				let scaled_path = self.scaled_movies_posters.join(&req_path);
// 				let full_path = self.posters_dir.join(&req_path);

// 				// scale image if it is missing
// 				scale_if_missing(scaled_path.clone(), full_path).await?;

// 				scaled_path
// 			} else if p.starts_with(POSTERS_SERIES_DIR) {
// 				// we need to transform series.jpg to series/poster.jpg
// 				let req_path = into_path_buf(&p[POSTERS_SERIES_DIR.len()..])?;

// 				let scaled_path = self.scaled_series_posters.join(&req_path);
// 				let mut full_path = self.series_dir.join(req_path);
// 				series_poster_transform(&mut full_path)?;

// 				// scale image if it is missing
// 				scale_if_missing(scaled_path.clone(), full_path).await?;

// 				scaled_path
// 			} else if p.starts_with(FULL_POSTERS_MOVIES_DIR) {
// 				let req_path =
// 					into_path_buf(&p[FULL_POSTERS_MOVIES_DIR.len()..])?;
// 				self.posters_dir.join(req_path)
// 			} else if p.starts_with(FULL_POSTERS_SERIES_DIR) {
// 				// we need to transform series.jpg to series/poster.jpg
// 				let req_path =
// 					into_path_buf(&p[FULL_POSTERS_SERIES_DIR.len()..])?;

// 				let mut path = self.series_dir.join(req_path);
// 				series_poster_transform(&mut path)?;

// 				path
// 			} else {
// 				unreachable!()
// 			};

// 			serve_file(file_path, &req, Some(self.caching.clone()))
// 				.await
// 				.map_err(Error::from_client_io)
// 		})
// 	}
// }

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

pub(crate) fn add_routes(server: &mut FireBuilder) {
	server.add_route(get_movies);
	server.add_route(get_posters_movies);
	server.add_route(get_full_posters_movies);
	server.add_route(get_series);
	server.add_route(get_posters_series);
	server.add_route(get_full_posters_series);
}
