import { outFunction } from "../outFunction.ts";
const externalVar = "World";

export function GET() {
  return new Response("Hello " + externalVar + outFunction(1, 2));
}

const completeText = "Hello " + externalVar;

export default () => {new Response(completeText);}
