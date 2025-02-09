<script lang="ts">
let { table_id } = $props()
type Text = string;
type Integer = number;
type Decimal = number;
type Money = Decimal;
type Progress = number;
type DateTime = Date;
type Interval = null;
type Weblink = string;
type Email = string;
type Checkbox = boolean
type Enumeration = number;
type Image = null;
type File = null;

type Cell = Text | Integer | Decimal | Money | Progress | DateTime | Interval | Weblink | Email | Checkbox | Enumeration | Image | File;

type Field = {
  name: string;
  options?: null;
};

type DataTable = {
  fields: Field[];
  rows: Cell[][];
};

const table: DataTable = {
  fields: [
    {
      name: "Project Name"
    },
    {
      name: "Funding"
    },
    {
      name: "Members"
    },
    {
      name: "Progress"
    }
  ],

  rows: [
    [
      "Project Alpha" as Text,
      30000 as Money,
      1 as Integer,
      60 as Progress
    ],
    [
      "Project Beta" as Text,
      40000 as Money,
      2 as Integer,
      20 as Progress
    ]
  ]
}
</script>

<table class=" border border-gray-400 border-spacing-1 bg-white text-black rounded-xl ">
  <thead>
    <tr>
    {#each table.fields as field}
      <th class="bg-gray-200 p-1 border-2 border-gray-400">{field.name}</th>
    {/each}
    </tr>
  </thead>
  <tbody>
    {#each table.rows as row}
      <tr>
        {#each row as cell}
          <td class="border-2 border-gray-400 bg-white p-1">{cell}</td>
        {/each}
      </tr>
    {/each}
  </tbody>

</table> 
