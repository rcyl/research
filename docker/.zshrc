if type brew &>/dev/null; then
    FPATH=$(brew --prefix)/share/zsh-completions:$FPATH

    autoload -Uz compinit
    compinit
fi

# Runs the gemini image with the credentials mounted in
podgemini() {
    podman run -it --rm \
            -v $(pwd)/:/src -w/src \
            -v "$HOME/.gemini:/root/.gemini" \
            gemini bash
}

# Runs the claude image with the credentials mounted in
podclaude() {
    podman run -it --rm \
            -v $(pwd)/:/src -w/src \
            -v "$HOME/.claude:/root/.claude" \
            -v "$HOME/.claude.json:/root/.claude.json" \
            claude bash
}

# Build a container in the current dir
podbuild() {
    if [[ -z "$1" ]]; then
        echo "Usage: podbuild <image_name>"
        return 1
    fi
	
	local image="$1"
	shift

    podman build -t "$image" .
}

# Run a container with defaults
podrun() {
    if [[ -z "$1" ]]; then
        echo "Usage: podrun <image_name>"
        return 1
    fi
	
	local image="$1"
    local name="$1"
	shift

    echo "Starting container: $name (Image: $image)"

    podman run -it --rm \
        --name "$name" \
		--tz=local \
		-v "$PWD:/src" -w/src \
        "$image"
}

# Exec into a running container
podexec() {
    
	if [[ -z "$1" ]]; then
        echo "Usage: podexec <container_name>"
        return 1
    fi

    local name="$1"
    local shell="${2:-/bin/bash}" # Default to bash, fallback to sh if needed

    echo "Entering container: $name..."
    
    # -i: interactive, -t: terminal
    podman exec -it "$name" "$shell"
}

# Load the version control system module
autoload -Uz vcs_info
precmd() { vcs_info }

# Format the output: %b is the branch name
# The 'zsh: ' prefix is just a label, you can change it to anything
zstyle ':vcs_info:git:*' formats '(%b)'

# Enable variable substitution in the prompt
setopt PROMPT_SUBST

# Set your prompt to include the vcs_info_msg_0_ variable
PROMPT='%F{cyan}%n@%m%f:%F{blue}%~%f %F{yellow}${vcs_info_msg_0_}%f %# '

. "$HOME/.local/bin/env"
