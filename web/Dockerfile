# Production image for the web app (see deploy/compose.yaml). The build stage
# runs natively on the build machine: its output is plain JavaScript, so an
# x86_64 image builds on an arm64 machine without emulation.
FROM --platform=$BUILDPLATFORM node:24.22.0-trixie-slim@sha256:7f486cabe892bff8fef04c0f7da8dba84a560be3e4e2740b7a86afc57ebec315 AS build

ENV COREPACK_ENABLE_DOWNLOAD_PROMPT=0
# pnpm is provided through corepack, pinned by package.json's packageManager field.
RUN corepack enable pnpm

WORKDIR /web
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml .npmrc ./
RUN pnpm install --frozen-lockfile
COPY . .
RUN pnpm build

# adapter-node bundles everything the server needs into build/: the image
# holds only Node and that output, and runs as the node user.
FROM node:24.22.0-trixie-slim@sha256:7f486cabe892bff8fef04c0f7da8dba84a560be3e4e2740b7a86afc57ebec315

WORKDIR /web
COPY --from=build /web/build build
COPY --from=build /web/package.json package.json

USER node
ENV NODE_ENV=production PORT=3000
EXPOSE 3000
CMD ["node", "build"]
