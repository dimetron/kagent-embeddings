

LOCALARCH ?= $(shell uname -m | sed -e 's/x86_64/amd64/' -e 's/aarch64/arm64/')

.PHONY: clean
clean:
	docker buildx prune --builder kagent-builder --force

.PHONY: build
build:
	docker buildx inspect --builder=kagent-builder || docker buildx create --name kagent-builder --use
	docker buildx build --builder kagent-builder --platform linux/$(LOCALARCH) --load -t kagent-embeddings .