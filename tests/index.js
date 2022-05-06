const { Worker } = require("worker_threads");

new Worker("./mint.js");
new Worker("./transfer1.js");
new Worker("./transfer2.js");
