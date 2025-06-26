

.PHONY: build
build:
	docker buildx inspect --builder=kagent-builder || docker buildx create --name kagent-builder --use
	docker buildx build --builder kagent-builder --load -t  kagent-embeddings .