import { expect, test } from 'vitest'
import { render } from 'vitest-browser-svelte'
import Login from '../../../src/routes/+page.svelte'


test('renders name', async () => {
  const screen = render(Login)
  await expect.element(screen.getByText('Chronicle', { exact: true })).toBeInTheDocument()
})
