# Server 

- Proxy path for stream 
- GET /v1/auth
  - Check if client has access to the stream.
  - Returns the user's uuid.
- POST /v1/user
  - session token and username is sent in the request body.
  - server will issue a cookie that contains access and refresh token that is to be used for the remainder of the session.
  - server will generate a random UUID to track the user, PUT /v1/user can be used to update the username.
- PUT /v1/user
  - Update the username
- POST /v1/chat
  - Check if client is banned. If banned, return 401.
  - Send message to all connected clients.
  - Persist message to store.
- GET /v1/chat
  - Return chat history from store, paginate if needed.
- WS /v1/im 
  - Websocket endpoint that broadcasts messages to clients. 
    Clients should never write to the server (except heartbeat)

# Frontend 

Write stuff here

