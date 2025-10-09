FROM alpine:3.22.2

COPY paperless-ngx-uploader /usr/bin/paperless-ngx-uploader

ENTRYPOINT [ "/usr/bin/paperless-ngx-uploader" ]