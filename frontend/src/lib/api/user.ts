import type { User, Credentials } from "$lib/types.js";
import { GET, POST, POST_FORM } from "./base.js";




export const postLogin = async (credentials: Credentials): Promise<User> => {
  let form = new FormData();
  form.append("username", credentials.username);
  form.append("password", credentials.password);
  return POST_FORM<User>(`/login`, form);
}

export const getLogout = async (): Promise<void> => GET(`/logout`);

export const getUser = async (): Promise<User> => GET<User>(`/user`);
