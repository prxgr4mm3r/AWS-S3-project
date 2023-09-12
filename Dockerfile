FROM ubuntu:20.04



RUN apt-get update
RUN apt-get install curl -y
RUN apt install build-essential -y
RUN apt install cargo -y
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

WORKDIR /app
COPY . /app

EXPOSE 8080
RUN apt update

RUN DEBIAN_FRONTEND=noninteractive apt install -y postgresql

RUN bash startUpScript.sh



