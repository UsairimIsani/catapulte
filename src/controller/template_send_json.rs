use crate::error::ServerError;
use crate::service::smtp::SmtpPool;
use crate::service::template::manager::TemplateManager;
use crate::service::template::provider::TemplateProvider;
use crate::service::template::template::TemplateOptions;
use actix_http::RequestHead;
use actix_web::{web, HttpResponse};
use lettre::Transport;

pub fn filter(req: &RequestHead) -> bool {
    req.headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Some(value == "application/json"))
        .unwrap_or(false)
}

pub async fn handler(
    smtp_pool: web::Data<SmtpPool>,
    template_provider: web::Data<TemplateProvider>,
    name: web::Path<String>,
    body: web::Json<TemplateOptions>,
) -> Result<HttpResponse, ServerError> {
    let template = template_provider.find_by_name(name.as_str())?;
    let email = template.to_email(&body)?;
    let mut conn = smtp_pool.get()?;
    conn.send(email)?;
    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use crate::tests::{create_email, execute_request, get_latest_inbox};
    use actix_web::http::StatusCode;
    use actix_web::test;
    use serde_json::json;

    #[actix_rt::test]
    #[serial]
    async fn success() {
        let from = create_email();
        let to = create_email();
        let payload = json!({
            "from": from.clone(),
            "to": to.clone(),
            "params": {
                "name": "bob",
                "token": "this_is_a_token"
            }
        });
        let req = test::TestRequest::post()
            .uri("/templates/user-login")
            .set_json(&payload)
            .to_request();
        let res = execute_request(req).await;
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        let list = get_latest_inbox(&from, &to).await;
        assert!(list.len() > 0);
        let last = list.first().unwrap();
        assert!(last.text.contains("Hello bob!"));
        assert!(last.html.contains("Hello bob!"));
        assert!(last
            .html
            .contains("\"http://example.com/login?token=this_is_a_token\""));
    }

    #[actix_rt::test]
    #[serial]
    async fn success_even_missing_params() {
        let from = create_email();
        let to = create_email();
        let payload = json!({
            "from": from.clone(),
            "to": to.clone(),
            "params": {
                "name": "bob"
            }
        });
        let req = test::TestRequest::post()
            .uri("/templates/user-login")
            .set_json(&payload)
            .to_request();
        let res = execute_request(req).await;
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        let list = get_latest_inbox(&from, &to).await;
        assert!(list.len() > 0);
        let last = list.first().unwrap();
        assert!(last.text.contains("Hello bob!"));
        assert!(last.html.contains("Hello bob!"));
        assert!(last.html.contains("\"http://example.com/login?token=\""));
    }

    #[actix_rt::test]
    #[serial]
    async fn failure_template_not_found() {
        let from = create_email();
        let to = create_email();
        let payload = json!({
            "from": from,
            "to": to,
            "params": {
                "name": "bob",
                "token": "this_is_a_token"
            }
        });
        let req = test::TestRequest::post()
            .uri("/templates/not-found")
            .set_json(&payload)
            .to_request();
        let res = execute_request(req).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    #[serial]
    async fn failure_invalid_arguments() {
        let from = create_email();
        let payload = json!({
            "from": from,
            "params": {
                "name": "bob",
                "token": "this_is_a_token"
            }
        });
        let req = test::TestRequest::post()
            .uri("/templates/user-login")
            .set_json(&payload)
            .to_request();
        let res = execute_request(req).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
