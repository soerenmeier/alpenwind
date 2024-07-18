use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Movie {
	id: u32,
	title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReducedMovie {
	adult: bool,
	backdrop_path: Option<String>,
	genre_ids: Vec<u32>,
	id: u32,
	original_language: String,
	original_title: String,
	overview: String,
	popularity: f32,
	poster_path: Option<String>,
	release_date: String,
	title: String,
	video: bool,
	vote_average: f32,
	vote_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieQuery {
	page: u32,
	total_pages: u32,
	total_results: u32,
	results: Vec<ReducedMovie>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Genre {
	id: u32,
	name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MovieGenresResponse {
	genres: Vec<Genre>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TmdbAsset {
	file_path: String,
	aspect_ratio: f32,
	width: u32,
	height: u32,
	iso_639_1: Option<String>,
	vote_average: f32,
	vote_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieImages {
	backdrops: Vec<TmdbAsset>,
	posters: Vec<TmdbAsset>,
	logos: Vec<TmdbAsset>,
}

#[derive(Debug)]
enum MovieError {
	Reqwest(reqwest::Error),
	NotFound,
	Unauthorized,
	Other,
}

struct TmdbClient {
	api_key: String,
	client: reqwest::Client,
}

impl TmdbClient {
	const BASE_URL: &'static str = "https://api.themoviedb.org/3";

	fn new(api_key: String) -> Result<Self, reqwest::Error> {
		let mut headers = header::HeaderMap::new();
		headers.insert(
			header::AUTHORIZATION,
			header::HeaderValue::from_str(&format!("Bearer {api_key}"))
				.unwrap(),
		);

		let client = reqwest::Client::builder()
			.default_headers(headers)
			.build()?;

		Ok(Self { api_key, client })
	}

	pub async fn get_movie(&self, id: u32) -> Result<Movie, MovieError> {
		let response = self
			.client
			.get(format!(
				"{BASE_URL}/movie/{id}",
				id = id,
				BASE_URL = Self::BASE_URL
			))
			.send()
			.await
			.map_err(MovieError::Reqwest)?;

		match response.status() {
			StatusCode::OK => {
				response.json::<Movie>().await.map_err(MovieError::Reqwest)
			}
			StatusCode::NOT_FOUND => Err(MovieError::NotFound),
			StatusCode::UNAUTHORIZED => Err(MovieError::Unauthorized),
			_ => Err(MovieError::Other),
		}
	}

	pub async fn search_movie(
		&self,
		query: &str,
		year: Option<u32>,
	) -> Result<MovieQuery, MovieError> {
		let mut url = format!(
			"{BASE_URL}/search/movie?query={query}",
			BASE_URL = Self::BASE_URL,
			query = query
		);

		if let Some(year) = year {
			url.push_str(&format!("&year={year}"));
		}

		let response = self
			.client
			.get(url)
			.send()
			.await
			.map_err(MovieError::Reqwest)?;

		match response.status() {
			StatusCode::OK => response
				.json::<MovieQuery>()
				.await
				.map_err(MovieError::Reqwest),
			StatusCode::NOT_FOUND => Err(MovieError::NotFound),
			StatusCode::UNAUTHORIZED => Err(MovieError::Unauthorized),
			_ => Err(MovieError::Other),
		}
	}
	pub async fn get_all_movie_genres(&self) -> Result<Vec<Genre>, MovieError> {
		let response = self
			.client
			.get(format!(
				"{BASE_URL}/genre/movie/list",
				BASE_URL = Self::BASE_URL
			))
			.send()
			.await
			.map_err(MovieError::Reqwest)?;

		match response.status() {
			StatusCode::OK => {
				let response = response
					.json::<MovieGenresResponse>()
					.await
					.map_err(MovieError::Reqwest)?;

				Ok(response.genres)
			}
			StatusCode::NOT_FOUND => Err(MovieError::NotFound),
			StatusCode::UNAUTHORIZED => Err(MovieError::Unauthorized),
			_ => Err(MovieError::Other),
		}
	}
	pub async fn get_movie_images(
		&self,
		id: u32,
	) -> Result<MovieImages, MovieError> {
		let response = self
			.client
			.get(format!(
				"{BASE_URL}/movie/{id}/images",
				id = id,
				BASE_URL = Self::BASE_URL
			))
			.send()
			.await
			.map_err(MovieError::Reqwest)?;

		match response.status() {
			StatusCode::OK => {
				let response = response
					.json::<MovieImages>()
					.await
					.map_err(MovieError::Reqwest)?;

				Ok(response)
			}
			StatusCode::NOT_FOUND => Err(MovieError::NotFound),
			StatusCode::UNAUTHORIZED => Err(MovieError::Unauthorized),
			_ => Err(MovieError::Other),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// get your own api key to run the tests
	const API_KEY: &str = "";

	#[tokio::test]
	async fn test_get_existing_movie() {
		let tmdb_client = TmdbClient::new(API_KEY.to_string()).unwrap();

		let movie = tmdb_client.get_movie(748783).await.unwrap();

		assert_eq!(
			movie,
			Movie {
				id: 748783,
				title: String::from("The Garfield Movie")
			}
		)
	}
	#[tokio::test]
	async fn test_searching_for_movie() {
		let tmdb_client = TmdbClient::new(API_KEY.to_string()).unwrap();

		let query = tmdb_client
			.search_movie("garfield", Some(2024))
			.await
			.unwrap();

		let m = query.results.first().unwrap();

		assert_eq!(m.id, 748783);
		assert_eq!(m.title, String::from("The Garfield Movie"));
	}

	#[tokio::test]
	async fn test_get_movie_genres() {
		let tmdb_client = TmdbClient::new(API_KEY.to_string()).unwrap();

		let genres = tmdb_client.get_all_movie_genres().await.unwrap();
		assert!(genres.iter().any(|g| g.name == "Action"));
	}

	#[tokio::test]
	async fn test_get_movie_images() {
		let tmdb_client = TmdbClient::new(API_KEY.to_string()).unwrap();

		let images = tmdb_client.get_movie_images(748783).await.unwrap();
		assert!(images.backdrops.len() > 0);
	}
}
