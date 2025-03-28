import { getLogout, postLogin } from "./api/login";
import type { User, Credentials } from "./types";

let userState: User | null = $state(null);

export const hasUser = () => user !== null;

export const user = () => userState;

export const login = (credentials: Credentials) => userState ? async () => { throw { body: "Already logged in." } } : postLogin(credentials).then((r: User) => {
  userState = r;
}).catch(e => {
  throw e;
})

export const logout = () => getLogout().then(() => userState = null);
