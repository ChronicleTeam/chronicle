


export type IsAdmin = boolean;


export type UserResponse = {
  user_id: number;      
  username: string;     
  is_admin: boolean;    
};


export type CreateUser = {
  username: string;
  password: string;
};


export type UpdateUser = {
  username?: string | null;
  password?: string | null;
};


export type SelectUser = {
  user_id: number;
};
