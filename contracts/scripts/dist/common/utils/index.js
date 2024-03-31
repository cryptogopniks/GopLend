import { AES, enc } from "crypto-js";
import axios from "axios";
const l = console.log.bind(console);
function li(object) {
  console.log(util.inspect(object, {
    showHidden: false,
    depth: null,
    colors: true
  }));
}
function getLast(arr) {
  return arr[arr.length - 1];
}
async function wait(delayInMilliseconds) {
  return new Promise(resolve => {
    setTimeout(resolve, delayInMilliseconds);
  });
}
class Request {
  constructor(config = {}) {
    this.req = axios.create(config);
  }
  async get(url, config) {
    return (await this.req.get(url, config)).data;
  }
  async post(url, params, config) {
    return (await this.req.post(url, params, config)).data;
  }
}
function encrypt(data, key) {
  return AES.encrypt(data, key).toString();
}
function decrypt(encryptedData, key) {
  // "Malformed UTF-8 data" workaround
  try {
    const bytes = AES.decrypt(encryptedData, key);
    return bytes.toString(enc.Utf8);
  } catch (error) {
    return;
  }
}
export { Request, l, li, getLast, wait, encrypt, decrypt };