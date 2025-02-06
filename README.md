# 🏆 CoC Dash

A simple **dashboard** for **Clash of Codes event** built with **Rust**, featuring a sleek UI for tracking battles, team standings, and player performance. **Clash of Codes** is a competitive programming event where participants solve coding challenges to earn points and improve their rankings.

## 🚀 Features

- 📊 **Battle Overview** – List all battles on the homepage.
- 🏅 **Team Standings** – View team rankings side by side.
- ⚔️ **Player Stats** – Track individual scores, ranks, and handles.
- 🔥 **Rust-Powered Backend** – Built with `actix_web` for fast and scalable performance.
- 🎨 **Rust-Based Frontend** – Designed using `Tera` for a seamless user experience.
- 📦 **Dockerized Deployment** – Easily deploy with persistent storage for battle data.
- 🌍 **Codeforces API Integration** – Fetches contest data dynamically to keep standings updated.

## 🛠 Tech Stack

- **Frontend:** Html & CSS
- **Backend:** Actix Web (Rust)
- **Templating:** Tera
- **Database:** JSON (Persistent Storage)
- **Containerization:** Docker
- **External API:** Codeforces API

## 🎮 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (version 1.83 or higher)
- [Docker](https://www.docker.com/)

### Setup

```sh
# Clone the repo
git clone https://github.com/Ferriccc/coc-dash.git
cd coc-dash

# Build and run the project
cargo run
```

### Run with Docker

```sh
docker build -t coc-dash .
docker run -p 8080:8080 coc-dash
```

