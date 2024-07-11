FROM rust:1.79.0-bullseye

# ARG USER_NAME=backer
# ARG USER_ID=1000
# ARG GROUP_ID=1000

# RUN addgroup -S -g ${GROUP_ID} ${USER_NAME} \
#     && adduser -u ${USER_ID} -G ${USER_NAME} -D ${USER_NAME}