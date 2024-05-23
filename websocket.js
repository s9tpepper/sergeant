const socket = new WebSocket("ws://127.0.0.1:54321");
// const socket = new WebSocket("ws://127.0.0.1:8080");

socket.addEventListener("message", (event) => {
  console.log(event.data);
});

socket.addEventListener("open", (event) => {
  console.log("socket is open");
  console.log(event);
});

socket.addEventListener("error", (event) => {
  console.log("socket had an error");
  console.log(event);
});

console.log("hello");
