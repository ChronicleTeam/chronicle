import { GET, DELETE, POST_FORM2, PATCH_FORM2 } from "./base.js";


export interface UserResponse {
  user_id: number;
  username: string;
  is_admin: boolean;
}


export interface CreateUser {
  username: string;
  password: string;
}

export interface UpdateUser {
  username?: string | null;
  password?: string | null;
}

export interface SelectUser {
  user_id: number;
}

/**
 * API calls
 */
export const getAllUsers = async (): Promise<UserResponse[]> =>
  GET<UserResponse[]>("/users");

export const createUser = async (payload: CreateUser): Promise<void> => {
  const params = new URLSearchParams();
  params.append("username", payload.username);
  params.append("password", payload.password);

  await POST_FORM2<void>("/users", params);
};


export const deleteUser = async (user_id: number): Promise<void> =>
  DELETE(`/users/${user_id}`);

export const updateUser = async (user_id: number, payload: UpdateUser): Promise<void> => {
  const params = new URLSearchParams();
  if (payload.username) params.append("username", payload.username);
  if (payload.password) params.append("password", payload.password);

  await PATCH_FORM2<void>(`/users/${user_id}`, params);
};



