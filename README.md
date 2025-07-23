"# Natural-Terrain-Generator"

Setup Guide
This system uses Rust for terrain generation and MongoDB for storing heightmaps. Docker is used to host MongoDB locally.
Below are the setup instructions to run the app

Prerequisites
Docker Desktop installed and running - [Install Docker](https://www.docker.com/products/docker-desktop/)

    MongoDB container running on localhost:27017 You should already have one running named fyp-mongo. If not, launch it:
        Bash: docker run -d --name fyp-mongo -p 27017:27017 mongo:latest

Running the App

1. Open your terminal.
2. Navigate to the root folder of the project.
3. Make sure MongoDB is running as described above.
4. Run the Rust app using cargo (release mode recommended):
   Bash: cargo run --release

Note
The app will attempt to connect to MongoDB using:
"mongodb://localhost:27017"
And access the database terrain_db with collection terrain2d

    If your app listens on a specific port (e.g. 3000), and that port is busy:
    Modify the host-port mapping in the Docker command like so:
        Bash: docker run -d --name fyp-mongo -p 27018:27017 mongo:latest
    Then change the Mongo URI in code accordingly:
    Bash: "mongodb://localhost:27018"
