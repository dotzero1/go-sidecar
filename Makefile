deploy:
	docker buildx build --platform linux/arm64 -t prod-sidecar -f docker/Dockerfile .
	./push_image.sh
