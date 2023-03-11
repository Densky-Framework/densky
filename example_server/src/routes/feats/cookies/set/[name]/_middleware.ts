import { HTTPError, HTTPRequest, IController, StatusCode } from "densky";

export default class _ implements IController {
  ANY(req: HTTPRequest) {
    if (!req.url.searchParams.has("data")) {
      return new HTTPError(StatusCode.BAD_REQUEST, "Missing 'data' param");
    }

    let data: unknown = req.url.searchParams.get("data")!;

    if (req.url.searchParams.has("type")) {
      const type = req.url.searchParams.get("type")!;
      if (type === "json") {
        data = JSON.parse(data as string);
      }
    }

    req.data.set("data", data);
  }
}
