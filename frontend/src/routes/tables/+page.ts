import type {PageLoad} from './$types';

export const load: PageLoad = async({fetch, params}) => {
  return { 
    tables: [
      {
        table_id:  0,
        user_id: 0,
        name: "Team",
        description: "The Team table",
        created_at: new Date()
      },{
        table_id:  1,
        user_id: 0,
        name: "Table 2",
        description: "Table 2 description",
        created_at: new Date()
      },{
        table_id:  2,
        user_id: 0,
        name: "Table 3",
        description: "Table 3 description",
        created_at: new Date()
      },
    ]
  }
}



