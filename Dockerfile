FROM alpine:3.24.1

# Reason: dockers_v2 organizes the build context by platform (e.g. linux/amd64/,
# linux/arm64/), so the binary must be copied from the $TARGETPLATFORM subdir.
ARG TARGETPLATFORM

COPY $TARGETPLATFORM/paperless-ngx-uploader /usr/bin/paperless-ngx-uploader

ENTRYPOINT [ "/usr/bin/paperless-ngx-uploader" ]
