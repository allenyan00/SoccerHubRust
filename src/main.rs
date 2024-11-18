use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use anyhow::{anyhow, Error};
use chrono::{DateTime, Utc};
use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use tera::{Context, Tera};
use yaml_rust::scanner::TokenType::Value;
use SoccerHubRust::config::get_data_from_url;

#[derive(Deserialize)]
struct QueryParams {
    competition_id: Option<String>,
    season: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ClubFlag {
    // 假设 logo_list 是一个 Vec<String>
    logo_list: Vec<TeamLogo>,

}
#[derive(Serialize, Deserialize, Clone)]
struct TeamLogo {
    id: u32,
    name: String,
    logo: String,
}

#[derive(Serialize, Deserialize)]
struct Standing {
    // 简化结构体，实际可能更复杂
    filters: Map<String, serde_json::Value>,
    area: Map<String, serde_json::Value>,
    competition: Map<String, serde_json::Value>,
    // 其他字段省略
    season : Map<String, serde_json::Value>,
    standings: Map<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct Team {
    name: String,
    logo: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Match {
    home_team: Team,
    away_team: Team,
    utc_date: String,
    date_time_format: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct CompetitionData {
    standings: Vec<Standing>,
    matches: Vec<Match>,
}

const API_KEY: &str = "d4c211a2fcbd4268b66b430969f34fbc";
const COMPETITIONS: &[&str] = &["PL", "LA", "BL"];


// lazy_static! {
//     static ref COMPETITIONS: HashMap<&'static str, &'static str> = {
//         let mut map = HashMap::new();
//         map.insert("Premier League", "PL");
//         map.insert("Bundesliga", "BL1");
//         map.insert("La Liga", "PD");
//         map.insert("Serie A", "SA");
//         map.insert("Ligue 1", "FL1");
//         map
//     };
// }

const SEASONS: &[i32] = &[2024, 2023, 2022];

fn get_client() -> Client {
    // 使用单例模式或依赖注入来重用 Client 实例
    Client::new()
}

async fn build_request(
    client: &Client,
    url: &str,
    headers: &[(String, String)],
    params: &[(String, String)],
) -> Result<reqwest::Response, Error> {
    // 验证 URL 和头信息
    if url.is_empty() || !url.starts_with("http") {
        return Err(anyhow!(StatusCode::BAD_REQUEST).context("Invalid URL"));
    }

    let mut header_map = HeaderMap::new();
    for (k, v) in headers {
        if k.is_empty() || v.is_empty() {
            return Err(anyhow!(StatusCode::BAD_REQUEST).context("Invalid header"));
        }
        let header_name = HeaderName::from_bytes(k.as_bytes()).map_err(|_| anyhow!(StatusCode::BAD_REQUEST).context("Invalid header name"))?;
        let header_value = HeaderValue::from_str(v).map_err(|_| anyhow!(StatusCode::BAD_REQUEST).context("Invalid header value"))?;
        header_map.insert(header_name, header_value);
    }

    // 构建请求
    let response = client
        .get(url)
        .headers(header_map)
        .query(params)
        .send()
        .await?;

    Ok(response)
}

async fn handle_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T, Error> {
    // 检查 HTTP 状态码
    if !response.status().is_success() {
        return Err(anyhow!(StatusCode::BAD_REQUEST).context("Request failed"));
    }

    // 解析 JSON
    let data: T = response.json().await?;
    Ok(data)
}

async fn get_club_flags(competition_id: &str, _season: i32) -> Result<Vec<TeamLogo>, Error> {
    let json_data = r#"
            [
            {
                "id": 57,
                "name": "Arsenal FC",
                "logo": "https://cdn.logoeye.net/club/fb/57.png"
            },
            {
                "id": 65,
                "name": "Manchester City FC",
                "logo": "https://cdn.logoeye.net/club/fb/65.png"
            },
            {
                "id": 64,
                "name": "Liverpool FC",
                "logo": "https://cdn.logoeye.net/club/fb/64.png"
            },
            {
                "id": 58,
                "name": "Aston Villa FC",
                "logo": "https://cdn.logoeye.net/club/fb/58.png"
            },
            {
                "id": 73,
                "name": "Tottenham Hotspur FC",
                "logo": "https://cdn.logoeye.net/club/fb/73.svg"
            },
            {
                "id": 67,
                "name": "Newcastle United FC",
                "logo": "https://cdn.logoeye.net/club/fb/67.png"
            },
            {
                "id": 61,
                "name": "Chelsea FC",
                "logo": "https://cdn.logoeye.net/club/fb/61.png"
            },
            {
                "id": 66,
                "name": "Manchester United FC",
                "logo": "https://cdn.logoeye.net/club/fb/66.png"
            },
            {
                "id": 563,
                "name": "West Ham United FC",
                "logo": "https://cdn.logoeye.net/club/fb/563.png"
            },
            {
                "id": 1044,
                "name": "AFC Bournemouth",
                "logo": "https://cdn.logoeye.net/club/fb/1044.png"
            },
            {
                "id": 397,
                "name": "Brighton \u0026 Hove Albion FC",
                "logo": "https://cdn.logoeye.net/club/fb/397.svg"
            },
            {
                "id": 76,
                "name": "Wolverhampton Wanderers FC",
                "logo": "https://cdn.logoeye.net/club/fb/76.svg"
            },
            {
                "id": 62,
                "name": "Everton FC",
                "logo": "https://cdn.logoeye.net/club/fb/62.png"
            },
            {
                "id": 63,
                "name": "Fulham FC",
                "logo": "https://cdn.logoeye.net/club/fb/63.svg"
            },
            {
                "id": 354,
                "name": "Crystal Palace FC",
                "logo": "https://cdn.logoeye.net/club/fb/354.png"
            },
            {
                "id": 402,
                "name": "Brentford FC",
                "logo": "https://cdn.logoeye.net/club/fb/402.png"
            },
            {
                "id": 351,
                "name": "Nottingham Forest FC",
                "logo": "https://cdn.logoeye.net/club/fb/351.png"
            },
            {
                "id": 389,
                "name": "Luton Town FC",
                "logo": "https://cdn.logoeye.net/club/fb/389.png"
            },
            {
                "id": 328,
                "name": "Burnley FC",
                "logo": "https://cdn.logoeye.net/club/fb/328.png"
            },
            {
                "id": 356,
                "name": "Sheffield United FC",
                "logo": "https://cdn.logoeye.net/club/fb/356.svg"
            },
            {
                "id": 338,
                "name": "Leicester City FC",
                "logo": "https://cdn.logoeye.net/club/fb/338.png"
            },
            {
                "id": 340,
                "name": "Southampton FC",
                "logo": "https://cdn.logoeye.net/club/fb/340.png"
            },
            {
                "id": 349,
                "name": "Ipswich Town FC",
                "logo": "https://cdn.logoeye.net/club/fb/349.png"
            }
        ]
    "#;
    
    // let json_data = r#"
    //     [{"id":57.0,"name":"Arsenal FC","logo":"https://cdn.logoeye.net/club/fb/57.png"},{"id":65.0,"name":"Manchester City FC","logo":"https://cdn.logoeye.net/club/fb/65.png"},{"id":64.0,"name":"Liverpool FC","logo":"https://cdn.logoeye.net/club/fb/64.png"},{"id":58.0,"name":"Aston Villa FC","logo":"https://cdn.logoeye.net/club/fb/58.png"},{"id":73.0,"name":"Tottenham Hotspur FC","logo":"https://cdn.logoeye.net/club/fb/73.svg"},{"id":67.0,"name":"Newcastle United FC","logo":"https://cdn.logoeye.net/club/fb/67.png"},{"id":61.0,"name":"Chelsea FC","logo":"https://cdn.logoeye.net/club/fb/61.png"},{"id":66.0,"name":"Manchester United FC","logo":"https://cdn.logoeye.net/club/fb/66.png"},{"id":563.0,"name":"West Ham United FC","logo":"https://cdn.logoeye.net/club/fb/563.png"},{"id":1044.0,"name":"AFC Bournemouth","logo":"https://cdn.logoeye.net/club/fb/1044.png"},{"id":397.0,"name":"Brighton \u0026 Hove Albion FC","logo":"https://cdn.logoeye.net/club/fb/397.svg"},{"id":76.0,"name":"Wolverhampton Wanderers FC","logo":"https://cdn.logoeye.net/club/fb/76.svg"},{"id":62.0,"name":"Everton FC","logo":"https://cdn.logoeye.net/club/fb/62.png"},{"id":63.0,"name":"Fulham FC","logo":"https://cdn.logoeye.net/club/fb/63.svg"},{"id":354.0,"name":"Crystal Palace FC","logo":"https://cdn.logoeye.net/club/fb/354.png"},{"id":402.0,"name":"Brentford FC","logo":"https://cdn.logoeye.net/club/fb/402.png"},{"id":351.0,"name":"Nottingham Forest FC","logo":"https://cdn.logoeye.net/club/fb/351.png"},{"id":389.0,"name":"Luton Town FC","logo":"https://cdn.logoeye.net/club/fb/389.png"},{"id":328.0,"name":"Burnley FC","logo":"https://cdn.logoeye.net/club/fb/328.png"},{"id":356.0,"name":"Sheffield United FC","logo":"https://cdn.logoeye.net/club/fb/356.svg"},{"id":338.0,"name":"Leicester City FC","logo":"https://cdn.logoeye.net/club/fb/338.png"},{"id":340.0,"name":"Southampton FC","logo":"https://cdn.logoeye.net/club/fb/340.png"},{"id":349.0,"name":"Ipswich Town FC","logo":"https://cdn.logoeye.net/club/fb/349.png"}]
    // "#;
    
    // let url = format!("https://cdn.logoeye.net/club/fb/list?region={}", competition_id);
    // let data = get_data_from_url(&url, &[], &[]).await;
    // if let Err(e) = data {
    //     let data_value = serde_json::Value::String(json_data.to_string());
    //     return Ok(serde_json::from_value(data_value)?);
    // }
    let club_flag: Vec<TeamLogo> = serde_json::from_str(json_data)?;
    Ok(club_flag)
    // let data_value = serde_json::Value::String(json_data.to_string());
    // Ok(serde_json::from_value(data_value)?)
}

async fn get_competition_standings(competition_id: &str, season: i32) -> Result<Standing, Error> {
    let url = format!("https://api.football-data.org/v4/competitions/{}/standings", competition_id);
    let headers = vec![("X-Auth-Token".to_string(), API_KEY.to_string())];
    let params = vec![("season".to_string(), season.to_string())];
    let data = get_data_from_url(&url, &headers, &params).await;
    let competition_data = serde_json::from_value(data?);
    Ok(competition_data?)
}

async fn get_competition_matches(competition_id: &str, season: i32) -> Result<CompetitionData, Error> {
    let url = format!("https://api.football-data.org/v4/competitions/{}/matches?status=SCHEDULED", competition_id);
    let headers = vec![("X-Auth-Token".to_string(), API_KEY.to_string())];
    let params = vec![("season".to_string(), season.to_string())];
    let data = get_data_from_url(&url, &headers, &params).await;
    let competition_data = serde_json::from_value(data?);
    Ok(competition_data?)
}

async fn get_completed_matches(competition_id: &str, season: i32) -> Result<CompetitionData, Error> {
    let url = format!("https://api.football-data.org/v4/competitions/{}/matches?status=FINISHED", competition_id);
    let headers = vec![("X-Auth-Token".to_string(), API_KEY.to_string())];
    let params = vec![("season".to_string(), season.to_string())];
    let data = get_data_from_url(&url, &headers, &params).await;
    let competition_data = serde_json::from_value(data?);
    Ok(competition_data?)
}

fn format_date_time(date_string: &str) -> Vec<String> {
    let date_object = DateTime::parse_from_rfc3339(date_string).unwrap().with_timezone(&Utc);
    vec![
        date_object.format("%d %b").to_string(),
        date_object.format("%H.%M").to_string(),
    ]
}

// fn set_logos_to_standings(standings: &mut Vec<Standing>, logo_list: &Vec<TeamLogo>) {
//     for standing in standings.iter_mut() {
//         if let Some(index) = logo_list.iter().position(|logo| logo.contains(&standing.team.name)) {
//             standing.team.logo = Some(logo_list[index].clone());
//         }
//     }
// }

fn set_logos_to_matches(matches: &mut Vec<Match>, logo_list: &Vec<String>) {
    for match_ in matches.iter_mut() {
        if let Some(index) = logo_list.iter().position(|logo| logo.contains(&match_.home_team.name)) {
            match_.home_team.logo = Some(logo_list[index].clone());
        }
        if let Some(index) = logo_list.iter().position(|logo| logo.contains(&match_.away_team.name)) {
            match_.away_team.logo = Some(logo_list[index].clone());
        }
        match_.date_time_format = Some(format_date_time(&match_.utc_date));
    }
}

fn set_date_time_completed_matches(matches: &mut Vec<Match>) {
    for match_ in matches.iter_mut() {
        match_.date_time_format = Some(format_date_time(&match_.utc_date));
    }
}

async fn home(query: web::Query<QueryParams>) -> impl Responder {
    let competition_id = query.competition_id.as_deref().unwrap_or("PL");
    let season = query.season.unwrap_or(2024);

    let club_flags = get_club_flags(competition_id, season).await.unwrap();
    let mut standings = get_competition_standings(competition_id, season).await.unwrap();
    let mut matches = get_competition_matches(competition_id, season).await.unwrap().matches;
    let mut completed_matches = get_completed_matches(competition_id, season).await.unwrap().matches;

    let current_date_time = format_date_time(&Utc::now().to_rfc3339());

    // set_logos_to_standings(&mut standings, &club_flags.logo_list);
    // set_logos_to_matches(&mut matches, &club_flags.logo_list);
    // set_logos_to_matches(&mut completed_matches, &club_flags);
    set_date_time_completed_matches(&mut completed_matches);

    let mut context = Context::new();
    context.insert("competitions", COMPETITIONS);
    context.insert("selected_competition", &competition_id);
    context.insert("seasons", &SEASONS);
    context.insert("selected_season", &season);
    context.insert("standings", &standings);
    context.insert("matches", &matches);
    context.insert("completed_matches", &completed_matches);
    context.insert("current_date_time", &current_date_time);

    let tera = Tera::new("templates/**/*").unwrap();
    let rendered = tera.render("index.html", &context).unwrap();

    HttpResponse::Ok().body(rendered)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(home))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

