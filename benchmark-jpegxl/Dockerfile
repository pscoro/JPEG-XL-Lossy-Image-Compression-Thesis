FROM ubuntu:latest

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
	cmake \
	clang \
	clang-format \
	clang-tidy \
	curl \
	doxygen \
	extra-cmake-modules \
	g++ \
	git \
	gcovr \
	graphviz \
	libavif-dev \
	libbenchmark-dev \
	libbenchmark-tools \
	libgif-dev \
	libgimp2.0-dev \
	libgoogle-perftools-dev \
	libgl1-mesa-dev \
	libgtest-dev \
	libgmock-dev \
	libjpeg-dev \
	libopenexr-dev \
	libpng-dev \
	libwebp-dev \
	ninja-build \
	parallel \
	pkg-config \
	qt6-base-dev \
	xdg-utils

WORKDIR /test-images
ADD ./test-image-set /test-images

WORKDIR /
RUN git clone https://github.com/libjxl/libjxl.git --recursive --shallow-submodules

WORKDIR /libjxl
RUN ./ci.sh opt

WORKDIR /temp
