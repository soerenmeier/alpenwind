## Build
FROM soerenmeier/chuchi-build as build

COPY --chown=build . .

RUN riji init_core_lib
RUN riji npm_ci
RUN riji build_all

## release
FROM soerenmeier/chuchi-release

COPY --chown=release --from=build /home/build/dist/ .

CMD ["./core-server", "--config", "/data/config.toml"]
