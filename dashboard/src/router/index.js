import { createRouter, createWebHistory } from 'vue-router'
import LiveView from '../views/LiveView.vue'

const routes = [
  {
    path: '/',
    name: 'live',
    component: LiveView
  },
  {
    path: '/history',
    name: 'history',
    component: () => import('../views/HistoryView.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
