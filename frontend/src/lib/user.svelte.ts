import { getLogout, getUser, postLogin } from "./api/user";
import type { User, Credentials } from "./types";

let userState: User | null = $state(null);

export const user = async () => {
  if (userState === null) {
    userState = await getUser().catch(() => null)
  }
  return userState
};

export const clearUser = async () => {
  userState = null;
}

export const login = (credentials: Credentials) => userState ? async () => { throw { body: "Already logged in." } } : postLogin(credentials).then((r: User) => {
  userState = r;
}).catch(e => {
  if (e.status === 400) { //already logged in
    return
  } else {
    throw e;
  }
})

export const logout = () => getLogout().then(() => userState = null);
