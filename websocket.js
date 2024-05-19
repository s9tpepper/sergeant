const socket = new WebSocket("ws://192.168.0.43:54321");

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
