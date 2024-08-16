import { AES, enc } from "crypto-js";
import util from "util";
import axios, {
  AxiosRequestConfig,
  AxiosInstance,
  CreateAxiosDefaults,
} from "axios";

const l = console.log.bind(console);

function li(object: any) {
  console.log(
    util.inspect(object, {
      showHidden: false,
      depth: null,
      colors: true,
    })
  );
}

function logAndReturn<T>(object: T, isDisplayed: boolean = false): T {
  if (isDisplayed) {
    l();
    li(object);
    l();
  }
  return object;
}

function floor(num: number, digits: number = 0): number {
  const k = 10 ** digits;
  return Math.floor(k * num) / k;
}

function getLast<T>(arr: T[]) {
  return arr[arr.length - 1];
}

async function wait(delayInMilliseconds: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, delayInMilliseconds);
  });
}

class Request {
  private req: AxiosInstance;

  constructor(config: CreateAxiosDefaults = {}) {
    this.req = axios.create(config);
  }

  async get<T>(url: string, config?: Object): Promise<T> {
    return (await this.req.get(url, config)).data;
  }

  async post(url: string, params: Object, config?: AxiosRequestConfig) {
    return (await this.req.post(url, params, config)).data;
  }
}

function encrypt(data: string, key: string): string {
  return AES.encrypt(data, key).toString();
}

function decrypt(encryptedData: string, key: string): string | undefined {
  // "Malformed UTF-8 data" workaround
  try {
    const bytes = AES.decrypt(encryptedData, key);
    return bytes.toString(enc.Utf8);
  } catch (error) {
    return;
  }
}

function getPaginationAmount(
  maxPaginationAmount: number,
  maxCount: number
): number {
  // limit maxPaginationAmount
  maxPaginationAmount = Math.min(
    maxPaginationAmount,
    maxCount || maxPaginationAmount
  );

  // update maxPaginationAmount to balance the load
  return maxCount
    ? Math.ceil(maxCount / Math.ceil(maxCount / maxPaginationAmount))
    : maxPaginationAmount;
}

export {
  Request,
  l,
  li,
  logAndReturn,
  floor,
  getLast,
  wait,
  encrypt,
  decrypt,
  getPaginationAmount,
};
