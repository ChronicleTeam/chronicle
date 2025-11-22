// User

export type Credentials = {
  username: string,
  password: string
}

export type User = {
  user_id: number,
  username: string
  is_admin?: boolean
}
