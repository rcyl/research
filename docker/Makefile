all:

bgemini:
	podman build -t gemini -f Dockerfile.gemini .

bclaude:
	podman build -t claude -f Dockerfile.claude .

gemini:
	podman run -it --rm \
		-v $$(pwd)/../:/src \
		-v $(HOME)/.gemini:/root/.gemini \
		gemini bash
claude:
	podman run -it --rm \
		-v $$(pwd)/../:/src \
		-v $(HOME)/.claude:/root/.claude \
		-v $(HOME)/.claude.json:/root/.claude.json \
		claude bash

