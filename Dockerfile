FROM alpine:3.24.0

COPY paperless-ngx-uploader /usr/bin/paperless-ngx-uploader

ENTRYPOINT [ "/usr/bin/paperless-ngx-uploader" ]