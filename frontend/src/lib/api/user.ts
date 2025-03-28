import type { User, Credentials } from "$lib/types.js";
import { GET, POST, POST_FORM } from "./base.js";




export const postLogin = async (credentials: Credentials): Promise<User> => POST_FORM<User>(`/login`, credentials);

export const getLogout = async (): Promise<void> => GET(`/logout`);

export const getUser = async (): Promise<User> => GET<User>(`/user`);
