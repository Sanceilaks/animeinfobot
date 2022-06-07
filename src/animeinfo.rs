use serenity::prelude::*;
use serenity::model::channel::Message;

const SHIKIAPIURL: &str = "https://shikimori.one/api/";
const ANILIBRIAAPIURL: &str = "http://anilibria.tv/public/api/index.php";

pub async fn find_animes(name: String, mut num: i32) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    num = num.min(10);
    let mut anime_ids: Vec<i64> = Vec::new();
    let resp = reqwest::get( std::format!("{}animes?limit={}&search={}", SHIKIAPIURL.to_string(), num, 
        urlencoding::encode(name.as_str()))).await?.json::<serde_json::Value>().await?;
    if resp.is_array() {
        for anime in resp.as_array().unwrap() {
            anime_ids.push(anime["id"].as_i64().unwrap());
        }
    }

    Ok(anime_ids)
}

pub async fn get_anime_info(anime_id: i64) -> Result<serde_json::Value, std::io::Error> {
    let resp = reqwest::get(SHIKIAPIURL.to_string() + "animes/" + &anime_id.to_string()).
        await.unwrap().json::<serde_json::Value>().await.unwrap();

    if resp["id"].is_i64() {
        Ok(resp)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Anime not found"))
    }
}

fn get_value_and_remove<T>(args: &mut Vec<&str>, prefix: &str, def: T) -> Result<T, Box<dyn std::error::Error>> 
    where 
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug
{
    let mut ret: T = def;
    args.iter().find( |f| f.contains(prefix) ).map(|f| {
        ret = f.split("=").nth(1).unwrap().parse::<T>().unwrap()
    });
    args.retain(|f| !f.contains(prefix));
    Ok(ret)
}

macro_rules! get_json_value {
    ($json:expr, $default:expr) => {
        if !$json.is_null() {
            $json.as_str().unwrap().to_string()
        } else {
            $default.to_string()
        }
    }
}

pub async fn anime_info(ctx: &Context, msg: &mut Message, mut args: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let output_num: i64 = get_value_and_remove(&mut args, "num=", 1).unwrap();
    let customid: i64 = get_value_and_remove(&mut args, "id=", 0).unwrap();

    if args.is_empty() && customid <= 0 {
        msg.edit(&ctx, |f| f.content("Не указано название аниме")).await?;
        return Ok(());
    }

    let mut animeids : Vec<i64> = Vec::new();
    
    if customid <= 0 {
        animeids = find_animes(args.join(" ").to_string(), output_num as i32).await?;
        if animeids.is_empty() {
            msg.edit(&ctx, |f| f.content("Аниме не найдено")).await?;
            return Ok(());
        }
    } else {
        animeids.push(customid);

    }

    if animeids.len() < 2 {
        for id in animeids {
            let _anime = get_anime_info(id).await;
            if _anime.is_err() {
                msg.edit(&ctx, |f| f.content("Аниме не найдено")).await?;
                return Ok(());
            }
            let anime = _anime.unwrap();

            let mut desc = format!("https://shikimori.one{}\n", 
                anime["url"].as_str().unwrap());
            if anime["description"].is_null() { 
                desc += "Описание отсутствует";
             } else {
                let html = anime["description"].as_str().unwrap().to_owned();
                desc += html2md::parse_html(html.as_str()).as_str();
            }

            let mut genres: Vec<String> = Vec::new();
            if anime["genres"].is_array() {
                for genre in anime["genres"].as_array().unwrap() {
                    if genre["russian"].is_string() {
                        genres.push(genre["russian"].as_str().unwrap().to_owned());
                    } else {
                        genres.push(genre["name"].as_str().unwrap().to_owned());
                    }
                    
                }
            }

            msg.edit(&ctx, |message|
                message.content("").add_embed(|emb|
                    emb.title(if anime["russian"].is_null() 
                            { anime["name"].as_str().unwrap() } else { anime["russian"].as_str().unwrap() }
                    ).
                    description(desc).
                    thumbnail(
                        if anime["image"].is_null()
                            { "https://i.imgur.com/5AXa838.jpeg".to_string() } 
                            else { "https://shikimori.one".to_string() + anime["image"]["preview"].as_str().unwrap() }
                    ).
                    field("Оценка", format!("{}/10", get_json_value!(anime["score"], "-")), true).
                    field("Статус", get_json_value!(anime["status"], "Неизвестно"), true).
                    field("Жанры", genres.join(", "), true)
                )
            ).await?;
        }
    } else {
        let mut animes: Vec<serde_json::Value> = Vec::new();
        for id in animeids {
            let animeinfo = get_anime_info(id).await?;
            animes.push(serde_json::json!({
                "name": if animeinfo["russian"].is_null() 
                    { animeinfo["name"].as_str().unwrap() } else { animeinfo["russian"].as_str().unwrap() },
                "id": id,
                "url" : "https://shikimori.one".to_string() + animeinfo["url"].as_str().unwrap()
            }));
        }

        msg.edit(&ctx, |message| {
            message.content("").embed( | emb | {
                emb.title("Результаты поиска");
                emb.description(animes.iter().map(|v| 
                    std::format!("<{}>[{}]({})", v["id"].as_i64().unwrap(), v["name"].as_str().unwrap(), v["url"].as_str().unwrap())
                ).collect::<Vec<String>>().join("\n"))
            })
        }).await?;
    }

    Ok(())
}