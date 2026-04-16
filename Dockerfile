FROM alpine:3.23.4

COPY paperless-ngx-uploader /usr/bin/paperless-ngx-uploader

ENTRYPOINT [ "/usr/bin/paperless-ngx-uploader" ]