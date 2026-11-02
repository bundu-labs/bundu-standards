// The register, read once at build time from this repository.
import raw from "../../../../register/register.yaml?raw";
import { parseRegister } from "./register.js";

export const REGISTER = parseRegister(raw);
