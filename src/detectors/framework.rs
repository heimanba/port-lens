use std::path::Path;

use crate::project_root::resolve_project_root;

/// Detected framework / runtime for a process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Framework {
    // Node.js
    NextJs,
    Vite,
    Express,
    Fastify,
    NestJs,
    Angular,
    Remix,
    Astro,
    SvelteKit,
    Nuxt,
    NodeGeneric,
    // Python
    Django,
    FastApi,
    Flask,
    Uvicorn,
    Gunicorn,
    PythonGeneric,
    // Ruby
    Rails,
    Sinatra,
    // Go
    GoGeneric,
    // Docker images
    PostgreSql,
    MySql,
    Redis,
    MongoDb,
    LocalStack,
    Nginx,
    Elasticsearch,
    RabbitMq,
    Kafka,
    DockerGeneric,
    // Unknown
    Unknown,
}

impl Framework {
    pub fn display_name(&self) -> &'static str {
        match self {
            Framework::NextJs => "Next.js",
            Framework::Vite => "Vite",
            Framework::Express => "Express",
            Framework::Fastify => "Fastify",
            Framework::NestJs => "NestJS",
            Framework::Angular => "Angular",
            Framework::Remix => "Remix",
            Framework::Astro => "Astro",
            Framework::SvelteKit => "SvelteKit",
            Framework::Nuxt => "Nuxt",
            Framework::NodeGeneric => "Node.js",
            Framework::Django => "Django",
            Framework::FastApi => "FastAPI",
            Framework::Flask => "Flask",
            Framework::Uvicorn => "uvicorn",
            Framework::Gunicorn => "gunicorn",
            Framework::PythonGeneric => "Python",
            Framework::Rails => "Rails",
            Framework::Sinatra => "Sinatra",
            Framework::GoGeneric => "Go",
            Framework::PostgreSql => "PostgreSQL",
            Framework::MySql => "MySQL",
            Framework::Redis => "Redis",
            Framework::MongoDb => "MongoDB",
            Framework::LocalStack => "LocalStack",
            Framework::Nginx => "nginx",
            Framework::Elasticsearch => "Elasticsearch",
            Framework::RabbitMq => "RabbitMQ",
            Framework::Kafka => "Kafka",
            Framework::DockerGeneric => "Docker",
            Framework::Unknown => "—",
        }
    }
}

/// Detect framework for a process.
/// Pass 1: read manifest files from `cwd`.
/// Pass 2: inspect `cmd` argv for known patterns.
pub fn detect(cmd: &[String], cwd: Option<&Path>) -> Framework {
    let cwd_for_files = cwd.map(|p| resolve_project_root(p).unwrap_or_else(|| p.to_path_buf()));
    if let Some(ref path) = cwd_for_files
        && let Some(fw) = detect_from_files(path)
    {
        return fw;
    }
    detect_from_cmd(cmd).unwrap_or(Framework::Unknown)
}

/// Detect a Docker image's framework by image name.
pub fn detect_docker_image(image: &str) -> Framework {
    let image = image.to_lowercase();
    if image.contains("postgres") {
        Framework::PostgreSql
    } else if image.contains("mysql") || image.contains("mariadb") {
        Framework::MySql
    } else if image.contains("redis") {
        Framework::Redis
    } else if image.contains("mongo") {
        Framework::MongoDb
    } else if image.contains("localstack") {
        Framework::LocalStack
    } else if image.contains("nginx") {
        Framework::Nginx
    } else if image.contains("elastic") {
        Framework::Elasticsearch
    } else if image.contains("rabbitmq") {
        Framework::RabbitMq
    } else if image.contains("kafka") || image.contains("zookeeper") {
        Framework::Kafka
    } else {
        Framework::DockerGeneric
    }
}

fn detect_from_files(cwd: &Path) -> Option<Framework> {
    // Node.js: inspect package.json
    let pkg_path = cwd.join("package.json");
    if pkg_path.exists()
        && let Ok(content) = std::fs::read_to_string(&pkg_path)
        && let Some(fw) = detect_from_package_json(&content)
    {
        return Some(fw);
    }

    // Python: pyproject.toml or requirements.txt
    if cwd.join("manage.py").exists() {
        return Some(Framework::Django);
    }
    if cwd.join("pyproject.toml").exists()
        && let Ok(content) = std::fs::read_to_string(cwd.join("pyproject.toml"))
    {
        if content.contains("fastapi") {
            return Some(Framework::FastApi);
        }
        if content.contains("flask") {
            return Some(Framework::Flask);
        }
        if content.contains("django") {
            return Some(Framework::Django);
        }
    }

    // Ruby: Gemfile
    if cwd.join("Gemfile").exists()
        && let Ok(content) = std::fs::read_to_string(cwd.join("Gemfile"))
    {
        if content.contains("rails") {
            return Some(Framework::Rails);
        }
        if content.contains("sinatra") {
            return Some(Framework::Sinatra);
        }
    }

    // Go: go.mod
    if cwd.join("go.mod").exists() {
        return Some(Framework::GoGeneric);
    }

    None
}

fn detect_from_package_json(content: &str) -> Option<Framework> {
    let json: serde_json::Value = serde_json::from_str(content).ok()?;

    let deps = {
        let mut all = serde_json::Map::new();
        if let Some(d) = json.get("dependencies").and_then(|v| v.as_object()) {
            all.extend(d.clone());
        }
        if let Some(d) = json.get("devDependencies").and_then(|v| v.as_object()) {
            all.extend(d.clone());
        }
        all
    };

    if deps.contains_key("next") {
        return Some(Framework::NextJs);
    }
    if deps.contains_key("vite") {
        return Some(Framework::Vite);
    }
    if deps.contains_key("@angular/core") {
        return Some(Framework::Angular);
    }
    if deps.contains_key("@remix-run/react") || deps.contains_key("@remix-run/node") {
        return Some(Framework::Remix);
    }
    if deps.contains_key("astro") {
        return Some(Framework::Astro);
    }
    if deps.contains_key("@sveltejs/kit") {
        return Some(Framework::SvelteKit);
    }
    if deps.contains_key("nuxt") {
        return Some(Framework::Nuxt);
    }
    if deps.contains_key("@nestjs/core") {
        return Some(Framework::NestJs);
    }
    if deps.contains_key("fastify") {
        return Some(Framework::Fastify);
    }
    if deps.contains_key("express") {
        return Some(Framework::Express);
    }

    Some(Framework::NodeGeneric)
}

fn detect_from_cmd(cmd: &[String]) -> Option<Framework> {
    let argv = cmd.join(" ").to_lowercase();

    // Python
    if argv.contains("uvicorn") {
        return Some(Framework::Uvicorn);
    }
    if argv.contains("gunicorn") {
        return Some(Framework::Gunicorn);
    }
    if argv.contains("manage.py") && argv.contains("runserver") {
        return Some(Framework::Django);
    }
    if argv.contains("flask") {
        return Some(Framework::Flask);
    }

    // Node.js
    if argv.contains("next") && (argv.contains("start") || argv.contains("dev")) {
        return Some(Framework::NextJs);
    }
    if argv.contains("vite") {
        return Some(Framework::Vite);
    }
    if argv.contains("remix") {
        return Some(Framework::Remix);
    }
    if argv.contains("astro") {
        return Some(Framework::Astro);
    }
    if argv.contains("nuxt") {
        return Some(Framework::Nuxt);
    }

    // Generic runtime fallback when we have command text but no specific framework signal.
    if argv.contains("node") {
        return Some(Framework::NodeGeneric);
    }
    if argv.contains("python") {
        return Some(Framework::PythonGeneric);
    }

    None
}
