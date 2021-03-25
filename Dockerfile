FROM node:14-alpine3.12

# Copy project to /app
COPY . /app
WORKDIR /app

# Install dependencies at build time
RUN yarn install --production

# At runtime
CMD ["yarn", "start"]
