FROM alpine:3.21.3

COPY paperless-ngx-uploader /usr/bin/paperless-ngx-uploader

ENTRYPOINT [ "/usr/bin/paperless-ngx-uploader" ]