echo "VITE_GIT_COMMIT_HASH=$(git rev-parse --short HEAD)" > .env
echo "VITE_GIT_COMMIT_BRANCH=$(git branch --show-current)" >> .env
